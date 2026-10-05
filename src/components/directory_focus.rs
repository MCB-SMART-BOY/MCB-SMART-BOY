use std::cell::{Cell, RefCell};

use leptos::{ev, html, prelude::*};
use wasm_bindgen::{JsCast, closure::Closure};

// Keep this threshold aligned with the named workspace query in site.css.
const CURRENT_DIRECTORY_MIN_WORKSPACE_PX: u16 = 1100;
const COMPACT_SELECTOR: &str = ".article-outline-compact";
const SUMMARY_SELECTOR: &str = ".article-outline-compact summary";

#[derive(Clone, Copy, PartialEq, Eq)]
enum DirectoryFocusOrigin {
    Rail,
    Compact,
}

#[derive(Clone, Copy)]
enum FocusDestination {
    Main,
    Responsive,
}

struct PendingFocus {
    destination: FocusDestination,
    handle: AnimationFrameRequestHandle,
}

#[derive(Clone, Copy)]
struct DirectoryFocusState {
    origin: StoredValue<Cell<Option<DirectoryFocusOrigin>>, LocalStorage>,
    pending: StoredValue<RefCell<Option<PendingFocus>>, LocalStorage>,
}

fn find_focus_origin(element: &web_sys::Element) -> Option<DirectoryFocusOrigin> {
    for (selector, origin) in [
        (".current-directory", DirectoryFocusOrigin::Rail),
        (COMPACT_SELECTOR, DirectoryFocusOrigin::Compact),
    ] {
        match element.closest(selector) {
            Ok(Some(_)) => return Some(origin),
            Ok(None) => {}
            Err(source) => web_sys::console::error_2(
                &format!("Failed to locate directory focus origin: {selector}").into(),
                &source,
            ),
        }
    }
    None
}

fn find_visible_destination(
    document: &web_sys::Document,
    selector: &str,
) -> Option<web_sys::HtmlElement> {
    match document.query_selector(selector) {
        Ok(target) => target
            .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
            .filter(|target| target.offset_width() > 0 && target.offset_height() > 0),
        Err(source) => {
            web_sys::console::error_2(
                &format!("Failed to find directory focus destination: {selector}").into(),
                &source,
            );
            None
        }
    }
}

fn focus_destination(destination: FocusDestination) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    // Resolve responsive requests at execution time, after the latest CSS layout.
    let selectors: &[&str] = match destination {
        FocusDestination::Main => &["#main"],
        FocusDestination::Responsive => &[".current-directory h2", SUMMARY_SELECTOR],
    };
    let Some((selector, target)) = selectors.iter().find_map(|selector| {
        find_visible_destination(&document, selector).map(|target| (*selector, target))
    }) else {
        return;
    };
    let target_origin = find_focus_origin(&target);
    if matches!(destination, FocusDestination::Responsive)
        && target_origin.is_some()
        && document
            .active_element()
            .as_ref()
            .and_then(find_focus_origin)
            == target_origin
    {
        return;
    }
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    if let Err(source) = target.focus_with_options(&options) {
        web_sys::console::error_2(
            &format!("Failed to focus directory destination: {selector}").into(),
            &source,
        );
    }
}

fn cancel_pending_focus(state: DirectoryFocusState) {
    if let Some(pending) = state
        .pending
        .with_value(|pending| pending.borrow_mut().take())
    {
        pending.handle.cancel();
    }
}

fn schedule_focus(state: DirectoryFocusState, destination: FocusDestination) {
    cancel_pending_focus(state);
    match request_animation_frame_with_handle(move || {
        state
            .pending
            .with_value(|pending| pending.borrow_mut().take());
        focus_destination(destination);
    }) {
        Ok(handle) => state.pending.with_value(|pending| {
            *pending.borrow_mut() = Some(PendingFocus {
                destination,
                handle,
            });
        }),
        Err(source) => web_sys::console::error_2(
            &"Failed to schedule directory focus transfer".into(),
            &source,
        ),
    }
}

fn watch_focus_origin(state: DirectoryFocusState) {
    let focus_listener = window_event_listener(ev::focusin, move |event| {
        let Some(element) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        else {
            return;
        };
        if matches!(element.tag_name().as_str(), "BODY" | "HTML") {
            return;
        }
        cancel_pending_focus(state);
        state
            .origin
            .with_value(|origin| origin.set(find_focus_origin(&element)));
    });
    let pointer_listener = window_event_listener(ev::pointerdown, move |event| {
        if event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|element| find_focus_origin(&element).is_none())
        {
            state.origin.with_value(|origin| origin.set(None));
            cancel_pending_focus(state);
        }
    });
    on_cleanup(move || {
        focus_listener.remove();
        pointer_listener.remove();
        cancel_pending_focus(state);
    });
}

fn watch_directory_routes(pathname: Memo<String>, state: DirectoryFocusState) {
    let initial_path = pathname.get_untracked();
    Effect::new(move |previous_path: Option<String>| {
        let path = pathname.get();
        if previous_path.as_deref().unwrap_or(initial_path.as_str()) != path
            && state.origin.with_value(|origin| origin.get().is_some())
        {
            schedule_focus(state, FocusDestination::Main);
        }
        path
    });
}

fn transfer_responsive_focus(state: DirectoryFocusState, is_wide: bool) {
    let pending = state
        .pending
        .with_value(|pending| pending.borrow().as_ref().map(|pending| pending.destination));
    if matches!(pending, Some(FocusDestination::Main)) {
        return;
    }
    let should_transfer = matches!(
        (state.origin.with_value(Cell::get), is_wide),
        (Some(DirectoryFocusOrigin::Rail), false) | (Some(DirectoryFocusOrigin::Compact), true)
    );
    if should_transfer || matches!(pending, Some(FocusDestination::Responsive)) {
        schedule_focus(state, FocusDestination::Responsive);
    }
}

fn install_directory_observer(element: web_sys::Element, state: DirectoryFocusState) {
    let was_wide = Cell::new(None::<bool>);
    let callback = Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::new(
        move |entries: js_sys::Array, _| {
            let Some(entry) = entries
                .get(0)
                .dyn_into::<web_sys::ResizeObserverEntry>()
                .ok()
            else {
                return;
            };
            let is_wide =
                entry.content_rect().width() >= f64::from(CURRENT_DIRECTORY_MIN_WORKSPACE_PX);
            if was_wide.replace(Some(is_wide)) != Some(is_wide) {
                transfer_responsive_focus(state, is_wide);
            }
        },
    );
    match web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
        Ok(observer) => {
            observer.observe(&element);
            let retained = StoredValue::new_local(RefCell::new(Some((observer, callback))));
            on_cleanup(move || {
                if let Some((observer, callback)) =
                    retained.with_value(|value| value.borrow_mut().take())
                {
                    observer.disconnect();
                    drop(callback);
                }
            });
        }
        Err(source) => web_sys::console::error_2(
            &"Failed to observe workspace width for directory focus".into(),
            &source,
        ),
    }
}

pub(super) fn watch_directory_focus(workspace: NodeRef<html::Div>, pathname: Memo<String>) {
    let state = DirectoryFocusState {
        origin: StoredValue::new_local(Cell::new(None)),
        pending: StoredValue::new_local(RefCell::new(None)),
    };
    watch_focus_origin(state);
    watch_directory_routes(pathname, state);
    Effect::new(move || {
        if let Some(workspace) = workspace.get() {
            install_directory_observer(workspace.into(), state);
        }
    });
}
