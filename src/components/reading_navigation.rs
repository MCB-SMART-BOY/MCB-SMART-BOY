use leptos::{html, prelude::*};
#[cfg(feature = "hydrate")]
use leptos_router::{NavigateOptions, hooks::use_navigate};
use leptos_router::{components::A, hooks::use_location};

use crate::{
    content::ROOT_DIRECTORY,
    locale::Locale,
    reading::{ReadingRoute, resolve_reading_route},
};

use super::{
    icons::{Icon, IconKind},
    reading_directory::ReadingTree,
    sidebar::SidebarNavigation,
};

#[cfg(feature = "hydrate")]
use leptos::ev;
#[cfg(feature = "hydrate")]
use std::cell::{Cell, RefCell};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure};

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum NavigationPage {
    Main,
    Reading,
}

impl NavigationPage {
    const COUNT: usize = 2;

    #[cfg(feature = "hydrate")]
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy)]
struct NavigationState {
    page: RwSignal<NavigationPage>,
    is_index: RwSignal<bool>,
    focus_request: RwSignal<Option<NavigationPage>>,
}

impl NavigationState {
    fn select_route(self, route: Option<ReadingRoute>, should_focus: bool) -> NavigationPage {
        let page = if route.is_some() {
            NavigationPage::Reading
        } else {
            NavigationPage::Main
        };
        self.is_index
            .set(matches!(route, Some(ReadingRoute::Index)));
        self.page.set(page);
        if should_focus {
            self.focus_request.set(Some(page));
        }
        page
    }
}

#[cfg(feature = "hydrate")]
fn navigate_root_page(
    state: NavigationState,
    page: NavigationPage,
    should_focus: bool,
    navigate: impl Fn(&str, NavigateOptions),
) {
    let (href, route) = match page {
        NavigationPage::Main => ("/", None),
        NavigationPage::Reading => (ROOT_DIRECTORY.path, Some(ReadingRoute::Index)),
    };
    state.select_route(route, should_focus);
    navigate(href, NavigateOptions::default());
}

pub(super) fn is_unmodified_click(event: &leptos::ev::MouseEvent) -> bool {
    !event.default_prevented()
        && event.button() == 0
        && !event.ctrl_key()
        && !event.meta_key()
        && !event.shift_key()
        && !event.alt_key()
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy, Default)]
struct WheelGesture {
    last_at: f64,
    locked_until: f64,
    accumulated: f64,
}

#[cfg(feature = "hydrate")]
const WHEEL_LINE_MODE: u32 = 1;
#[cfg(feature = "hydrate")]
const WHEEL_PAGE_MODE: u32 = 2;
#[cfg(feature = "hydrate")]
const WHEEL_LINE_PX: f64 = 16.0;
#[cfg(feature = "hydrate")]
const WHEEL_THRESHOLD_PX: f64 = 36.0;
#[cfg(feature = "hydrate")]
const WHEEL_GESTURE_GAP_MS: f64 = 180.0;
#[cfg(feature = "hydrate")]
const WHEEL_FLIP_LOCK_MS: f64 = 560.0;

#[cfg(feature = "hydrate")]
fn is_scrollable_in_direction(top: i32, height: i32, viewport: i32, delta: f64) -> bool {
    let remaining = height - viewport;
    remaining > 1
        && if delta > 0.0 {
            top < remaining - 1
        } else {
            top > 1
        }
}

#[cfg(feature = "hydrate")]
fn has_focus_within(panel: NodeRef<html::Div>) -> bool {
    let active = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element());
    active.is_some_and(|active| {
        panel
            .get_untracked()
            .is_some_and(|panel| panel.contains(Some(&active)))
    })
}

#[cfg(feature = "hydrate")]
fn handle_navigation_wheel(
    state: NavigationState,
    panel_ref: NodeRef<html::Div>,
    delta: f64,
    now: f64,
    gesture_cell: &Cell<WheelGesture>,
    navigate: impl Fn(&str, NavigateOptions),
) -> bool {
    let Some(panel) = panel_ref.get_untracked() else {
        return false;
    };
    let mut gesture = gesture_cell.get();
    if now < gesture.locked_until
        || (gesture.locked_until > 0.0 && now - gesture.last_at < WHEEL_GESTURE_GAP_MS)
    {
        gesture.last_at = now;
        gesture_cell.set(gesture);
        return true;
    }
    if is_scrollable_in_direction(
        panel.scroll_top(),
        panel.scroll_height(),
        panel.client_height(),
        delta,
    ) {
        gesture_cell.set(WheelGesture::default());
        return false;
    }
    gesture.locked_until = 0.0;
    if now - gesture.last_at > WHEEL_GESTURE_GAP_MS
        || gesture.accumulated.signum() != delta.signum()
    {
        gesture.accumulated = 0.0;
    }
    gesture.last_at = now;
    gesture.accumulated += delta;
    if gesture.accumulated.abs() >= WHEEL_THRESHOLD_PX {
        let next = if delta > 0.0 {
            NavigationPage::Reading
        } else {
            NavigationPage::Main
        };
        navigate_root_page(state, next, has_focus_within(panel_ref), &navigate);
        gesture.locked_until = now + WHEEL_FLIP_LOCK_MS;
        gesture.accumulated = 0.0;
    }
    gesture_cell.set(gesture);
    true
}

#[cfg(feature = "hydrate")]
fn is_interactive_key_target(event: &ev::KeyboardEvent) -> bool {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .is_some_and(|target| {
            target
                .closest("a[aria-expanded], button, input, select, textarea, [contenteditable]")
                .ok()
                .flatten()
                .is_some()
        })
}

#[cfg(feature = "hydrate")]
fn handle_navigation_key(
    event: &ev::KeyboardEvent,
    state: NavigationState,
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
    navigate: impl Fn(&str, NavigateOptions),
) {
    if event.default_prevented()
        || event.alt_key()
        || event.ctrl_key()
        || event.meta_key()
        || event.shift_key()
        || is_interactive_key_target(event)
    {
        return;
    }
    let page = state.page.get_untracked();
    let next = match (page, event.key().as_str()) {
        (NavigationPage::Main, "PageDown" | "ArrowDown") => NavigationPage::Reading,
        (NavigationPage::Reading, "PageUp" | "ArrowUp") if state.is_index.get_untracked() => {
            let Some(panel) = panels[NavigationPage::Reading.index()].get_untracked() else {
                return;
            };
            if panel.scroll_top() > 1 {
                return;
            }
            NavigationPage::Main
        }
        _ => return,
    };
    event.prevent_default();
    navigate_root_page(state, next, true, navigate);
}

#[cfg(feature = "hydrate")]
fn focus_heading(target: NodeRef<html::H2>) {
    let Some(element) = target.get_untracked() else {
        return;
    };
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    if let Err(source) = element.focus_with_options(&options) {
        web_sys::console::error_2(
            &"Failed to focus reading navigation heading".into(),
            &source,
        );
    }
}

#[cfg(feature = "hydrate")]
fn schedule_heading_focus(
    target: NodeRef<html::H2>,
    page: NavigationPage,
    state: NavigationState,
    pending: StoredValue<RefCell<Option<AnimationFrameRequestHandle>>, LocalStorage>,
) {
    if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
        handle.cancel();
    }
    match request_animation_frame_with_handle(move || {
        pending.with_value(|pending| {
            pending.borrow_mut().take();
        });
        if state.page.get_untracked() == page {
            focus_heading(target);
            state.focus_request.set(None);
        }
    }) {
        Ok(handle) => pending.with_value(|pending| *pending.borrow_mut() = Some(handle)),
        Err(source) => web_sys::console::error_2(
            &"Failed to schedule reading navigation focus".into(),
            &source,
        ),
    }
}

#[component]
fn ReadingPage(
    state: NavigationState,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
    is_hydrated: RwSignal<bool>,
    id_prefix: &'static str,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-reading"
            inert=move || state.page.get() != NavigationPage::Reading
            aria-hidden=move || if state.page.get() == NavigationPage::Reading { "false" } else { "true" }>
            <div class="navigation-page-header">
                <A attr:class="sidebar-link navigation-back" href="/" exact=true attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.select_route(None, true);
                        }
                    }>
                    <span aria-hidden="true">"←"</span>{move || locale.get().select("返回主导航", "Back to navigation")}
                </A>
                <h2 node_ref=heading tabindex="-1">{move || locale.get().select("阅读导航", "Reading navigation")}</h2>
                <A attr:class=move || if state.is_index.get() {
                    "sidebar-link reading-tree-overview is-active"
                } else {
                    "sidebar-link reading-tree-overview"
                } href=ROOT_DIRECTORY.path exact=true attr:data-sidebar-level="">
                    {move || locale.get().select("文章目录", "Writing directory")}
                </A>
            </div>
            <ReadingTree is_hydrated id_prefix/>
        </div>
    }
}

#[component]
fn MainPage(
    state: NavigationState,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-main"
            inert=move || state.page.get() != NavigationPage::Main
            aria-hidden=move || if state.page.get() == NavigationPage::Main { "false" } else { "true" }>
            <div class="navigation-page-header">
                <h2 node_ref=heading tabindex="-1">{move || locale.get().select("主导航", "Main navigation")}</h2>
            </div>
            <SidebarNavigation show_writing_link=false/>
            <div class="navigation-entry">
                <A attr:class="sidebar-link" href=ROOT_DIRECTORY.path attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.select_route(Some(ReadingRoute::Index), true);
                        }
                    }>
                    <Icon kind=IconKind::Writing/>{move || locale.get().select("文章目录", "Writing")}</A>
                <p>{move || locale.get().select("向下滚动浏览文章目录", "Scroll down to browse writing")}</p>
            </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn handle_wheel_input(
    event: &ev::WheelEvent,
    state: NavigationState,
    is_visible: Signal<bool>,
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
    gesture: &Cell<WheelGesture>,
    navigate: impl Fn(&str, NavigateOptions),
) {
    if !is_visible.get_untracked()
        || event.ctrl_key()
        || event.meta_key()
        || event.alt_key()
        || event.shift_key()
    {
        return;
    }
    let page = state.page.get_untracked();
    let panel = panels[page.index()];
    let delta = match event.delta_mode() {
        WHEEL_LINE_MODE => event.delta_y() * WHEEL_LINE_PX,
        WHEEL_PAGE_MODE => {
            event.delta_y()
                * f64::from(
                    panel
                        .get_untracked()
                        .map_or(0, |panel| panel.client_height()),
                )
        }
        _ => event.delta_y(),
    };
    if delta.abs() < f64::EPSILON {
        return;
    }
    if page == NavigationPage::Reading && !state.is_index.get_untracked() {
        return;
    }
    let gesture_locked = gesture.get().locked_until > event.time_stamp();
    if !gesture_locked
        && ((page == NavigationPage::Main && delta < 0.0)
            || (page == NavigationPage::Reading && delta > 0.0))
    {
        return;
    }
    if handle_navigation_wheel(state, panel, delta, event.time_stamp(), gesture, navigate) {
        event.prevent_default();
    }
}

#[cfg(feature = "hydrate")]
type WheelListener = (web_sys::HtmlElement, Closure<dyn FnMut(ev::WheelEvent)>);

#[cfg(feature = "hydrate")]
fn remove_wheel_listener((element, callback): WheelListener) {
    if let Err(source) =
        element.remove_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())
    {
        web_sys::console::error_2(
            &"Failed to remove reading navigation wheel listener".into(),
            &source,
        );
    }
}

#[cfg(feature = "hydrate")]
fn install_navigation_wheel(
    container: NodeRef<html::Div>,
    state: NavigationState,
    is_visible: Signal<bool>,
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
    navigate: impl Fn(&str, NavigateOptions) + Clone + 'static,
) {
    let listener = StoredValue::new_local(RefCell::new(None::<WheelListener>));
    let gesture = StoredValue::new_local(Cell::new(WheelGesture::default()));
    Effect::new(move |_| {
        if let Some(previous) = listener.with_value(|listener| listener.borrow_mut().take()) {
            remove_wheel_listener(previous);
        }
        let Some(element) = container.get() else {
            return;
        };
        let element: web_sys::HtmlElement = element.into();
        let navigate = navigate.clone();
        let callback = Closure::<dyn FnMut(ev::WheelEvent)>::new(move |event| {
            gesture.with_value(|gesture| {
                handle_wheel_input(&event, state, is_visible, panels, gesture, &navigate)
            });
        });
        let options = web_sys::AddEventListenerOptions::new();
        options.set_passive(false);
        if let Err(source) = element
            .add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                callback.as_ref().unchecked_ref(),
                &options,
            )
        {
            web_sys::console::error_2(
                &"Failed to attach reading navigation wheel listener".into(),
                &source,
            );
            return;
        }
        listener.with_value(|listener| *listener.borrow_mut() = Some((element, callback)));
    });
    on_cleanup(move || {
        if let Some(previous) = listener.with_value(|listener| listener.borrow_mut().take()) {
            remove_wheel_listener(previous);
        }
    });
}

#[derive(Clone, Copy)]
struct NavigationNodes {
    headings: [NodeRef<html::H2>; NavigationPage::COUNT],
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
}

#[cfg(feature = "hydrate")]
fn watch_navigation_updates(
    pathname: Memo<String>,
    initial_path: String,
    reading_request: RwSignal<u64>,
    state: NavigationState,
    is_visible: Signal<bool>,
    nodes: NavigationNodes,
    navigate: impl Fn(&str, NavigateOptions) + 'static,
) {
    Effect::new(move |previous_path: Option<String>| {
        let path = pathname.get();
        if previous_path.as_deref().unwrap_or(initial_path.as_str()) != path {
            let route = resolve_reading_route(&path);
            let next_page = if route.is_some() {
                NavigationPage::Reading
            } else {
                NavigationPage::Main
            };
            let previous_page = state.page.get_untracked();
            let had_sidebar_focus = previous_page != next_page
                && is_visible.get_untracked()
                && has_focus_within(nodes.panels[previous_page.index()]);
            let page = state.select_route(route, had_sidebar_focus);
            if !had_sidebar_focus && state.focus_request.get_untracked() != Some(page) {
                state.focus_request.set(None);
            }
        }
        path
    });
    Effect::new(move |previous_request: Option<u64>| {
        let request = reading_request.get();
        if previous_request.is_some_and(|previous| previous != request) {
            state.select_route(Some(ReadingRoute::Index), true);
            if pathname.get_untracked() != ROOT_DIRECTORY.path {
                navigate(ROOT_DIRECTORY.path, NavigateOptions::default());
            }
        }
        request
    });
    let pending = StoredValue::new_local(RefCell::new(None::<AnimationFrameRequestHandle>));
    Effect::new(move |_| {
        let target = state.focus_request.get();
        if !is_visible.get() || target.is_none() {
            if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
                handle.cancel();
            }
            return;
        }
        let page = target.unwrap_or(NavigationPage::Main);
        schedule_heading_focus(nodes.headings[page.index()], page, state, pending);
    });
    on_cleanup(move || {
        if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
            handle.cancel();
        }
    });
}

#[component]
pub(crate) fn ReadingNavigation(
    is_hydrated: RwSignal<bool>,
    is_visible: Signal<bool>,
    reading_request: RwSignal<u64>,
    id_prefix: &'static str,
) -> impl IntoView {
    let pathname = use_location().pathname;
    let initial_path = pathname.get_untracked();
    let state = NavigationState {
        page: RwSignal::new(NavigationPage::Main),
        is_index: RwSignal::new(false),
        focus_request: RwSignal::new(None),
    };
    state.select_route(resolve_reading_route(&initial_path), false);
    let nodes = NavigationNodes {
        headings: std::array::from_fn(|_| NodeRef::new()),
        panels: std::array::from_fn(|_| NodeRef::new()),
    };
    #[cfg(feature = "hydrate")]
    watch_navigation_updates(
        pathname,
        initial_path,
        reading_request,
        state,
        is_visible,
        nodes,
        use_navigate(),
    );
    #[cfg(not(feature = "hydrate"))]
    let _ = (initial_path, reading_request);
    view! {
        <NavigationSurface state is_visible nodes is_hydrated id_prefix/>
    }
}

#[component]
fn NavigationSurface(
    state: NavigationState,
    is_visible: Signal<bool>,
    nodes: NavigationNodes,
    is_hydrated: RwSignal<bool>,
    id_prefix: &'static str,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let [main_heading, reading_heading] = nodes.headings;
    let [main_panel, reading_panel] = nodes.panels;
    let container = NodeRef::<html::Div>::new();
    #[cfg(feature = "hydrate")]
    let navigate = use_navigate();
    #[cfg(feature = "hydrate")]
    install_navigation_wheel(container, state, is_visible, nodes.panels, navigate.clone());
    #[cfg(not(feature = "hydrate"))]
    let _ = is_visible;
    view! {
        <div node_ref=container class="reading-navigation" role="region"
            aria-label=move || locale.get().select("阅读导航", "Reading navigation")
            on:keydown=move |event| {
                #[cfg(feature = "hydrate")]
                handle_navigation_key(&event, state, nodes.panels, &navigate);
                #[cfg(not(feature = "hydrate"))]
                let _ = event;
            }>
            <div class="reading-navigation-track"
                class:page-main=move || state.page.get() == NavigationPage::Main
                class:page-reading=move || state.page.get() == NavigationPage::Reading>
                <MainPage state heading=main_heading panel=main_panel/>
                <ReadingPage state heading=reading_heading panel=reading_panel is_hydrated id_prefix/>
            </div>
        </div>
    }
}
