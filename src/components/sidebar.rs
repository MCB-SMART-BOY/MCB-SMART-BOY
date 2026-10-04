use leptos::{html, prelude::*};
use leptos_router::{components::A, hooks::use_location};

#[cfg(feature = "hydrate")]
use crate::preferences::persist_preferences;
use crate::{
    locale::Locale,
    preferences::{
        SIDEBAR_CONTENT_MIN_PX, SIDEBAR_MAX_PX, SIDEBAR_MIN_PX, SIDEBAR_MOBILE_BREAKPOINT_PX,
        UiPreferences,
    },
};

use super::{
    icons::{Icon, IconKind},
    reading_navigation::ReadingNavigation,
};

#[cfg(feature = "hydrate")]
use std::cell::RefCell;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
struct DragState {
    pointer_id: i32,
    start_x: i32,
    initial_width: u16,
    initial_visible_width: u16,
}

fn available_max(viewport_width: u16) -> u16 {
    viewport_width
        .saturating_sub(SIDEBAR_CONTENT_MIN_PX)
        .clamp(SIDEBAR_MIN_PX, SIDEBAR_MAX_PX)
}

pub(super) fn save_preferences(preferences: UiPreferences) {
    #[cfg(feature = "hydrate")]
    persist_preferences(preferences);
    #[cfg(not(feature = "hydrate"))]
    let _ = preferences;
}

#[cfg(feature = "hydrate")]
fn cancel_brand_focus(
    pending: StoredValue<RefCell<Option<AnimationFrameRequestHandle>>, LocalStorage>,
) {
    if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
        handle.cancel();
    }
}

#[cfg(feature = "hydrate")]
fn focus_brand_target(is_collapsed: bool) {
    let selector = if is_collapsed {
        ".site-sidebar .brand-expand"
    } else {
        ".site-sidebar .sidebar-brand a.brand"
    };
    let target = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| match document.query_selector(selector) {
            Ok(target) => target,
            Err(source) => {
                web_sys::console::error_2(
                    &"Failed to find sidebar brand focus target".into(),
                    &source,
                );
                None
            }
        })
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    let Some(target) = target else {
        web_sys::console::error_1(&"Sidebar brand focus target is unavailable".into());
        return;
    };
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    if let Err(source) = target.focus_with_options(&options) {
        web_sys::console::error_2(&"Failed to focus sidebar brand".into(), &source);
    }
}

#[cfg(feature = "hydrate")]
fn schedule_brand_focus(
    is_collapsed: bool,
    pending: StoredValue<RefCell<Option<AnimationFrameRequestHandle>>, LocalStorage>,
) {
    cancel_brand_focus(pending);
    let callback_pending = pending;
    match request_animation_frame_with_handle(move || {
        callback_pending.with_value(|pending| {
            pending.borrow_mut().take();
        });
        focus_brand_target(is_collapsed);
    }) {
        Ok(handle) => pending.with_value(|pending| *pending.borrow_mut() = Some(handle)),
        Err(source) => {
            web_sys::console::error_2(&"Failed to schedule sidebar brand focus".into(), &source)
        }
    }
}

#[cfg(feature = "hydrate")]
fn capture_brand_flight(
    brand: NodeRef<html::Div>,
    collapse: NodeRef<html::Button>,
    mark: NodeRef<html::Span>,
) -> bool {
    let Some((brand, collapse, mark)) = brand
        .get_untracked()
        .zip(collapse.get_untracked())
        .zip(mark.get_untracked())
        .map(|((brand, collapse), mark)| (brand, collapse, mark))
    else {
        web_sys::console::error_1(&"Cannot measure sidebar brand flight: missing element".into());
        return false;
    };
    let brand: web_sys::HtmlElement = brand.into();
    let container = brand.get_bounding_client_rect();
    let source = collapse.get_bounding_client_rect();
    let target = mark.get_bounding_client_rect();
    let sizes = [
        container.width(),
        container.height(),
        source.width(),
        source.height(),
        target.width(),
        target.height(),
    ];
    if sizes.iter().any(|size| !size.is_finite() || *size <= 0.0) {
        web_sys::console::error_1(
            &"Cannot measure sidebar brand flight: invalid element size".into(),
        );
        return false;
    }
    let source_x = source.x() + source.width() / 2.0 - container.x();
    let source_y = source.y() + source.height() / 2.0 - container.y();
    let target_x = target.x() + target.width() / 2.0 - container.x();
    let target_y = target.y() + target.height() / 2.0 - container.y();
    for (name, value) in [
        ("--brand-flight-x", source_x),
        ("--brand-flight-y", source_y),
        ("--brand-flight-dx", target_x - source_x),
        ("--brand-flight-dy", target_y - source_y),
    ] {
        if !value.is_finite() {
            web_sys::console::error_1(
                &"Cannot measure sidebar brand flight: invalid position".into(),
            );
            return false;
        }
        if let Err(source) = brand.style().set_property(name, &format!("{value}px")) {
            web_sys::console::error_2(&"Cannot set sidebar brand flight position".into(), &source);
            return false;
        }
    }
    true
}

#[cfg(feature = "hydrate")]
fn should_animate_brand() -> bool {
    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(
            &"Cannot check sidebar motion preference: window unavailable".into(),
        );
        return false;
    };
    match window.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(query)) => !query.matches(),
        Ok(None) => {
            web_sys::console::error_1(
                &"Cannot check sidebar motion preference: no media query result".into(),
            );
            false
        }
        Err(source) => {
            web_sys::console::error_2(&"Failed to check sidebar motion preference".into(), &source);
            false
        }
    }
}

#[derive(Clone, Copy)]
struct BrandState {
    is_hydrated: RwSignal<bool>,
    is_desktop: bool,
    preferences: RwSignal<UiPreferences>,
    locale: RwSignal<Locale>,
    is_collapsing: RwSignal<bool>,
    brand: NodeRef<html::Div>,
    collapse: NodeRef<html::Button>,
    mark: NodeRef<html::Span>,
    #[cfg(feature = "hydrate")]
    pending_focus: StoredValue<RefCell<Option<AnimationFrameRequestHandle>>, LocalStorage>,
}

fn render_brand_logo(state: BrandState) -> impl IntoView {
    move || {
        if state.is_desktop && state.preferences.get().is_sidebar_collapsed {
            view! {
                <button class="brand brand-expand" type="button"
                    disabled=move || !state.is_hydrated.get()
                    aria-controls="site-sidebar" aria-expanded="false"
                    aria-label=move || state.locale.get().select("展开侧栏", "Expand sidebar")
                    title=move || state.locale.get().select("展开侧栏", "Expand sidebar")
                    on:click=move |_| {
                        state.is_collapsing.set(false);
                        state.preferences.update(|value| value.is_sidebar_collapsed = false);
                        save_preferences(state.preferences.get_untracked());
                        #[cfg(feature = "hydrate")]
                        schedule_brand_focus(false, state.pending_focus);
                    }>
                    <span class="brand-mark" aria-hidden="true">"M"<span class="brand-mark-dot">"."</span></span>
                    <Show when=move || state.is_collapsing.get()>
                        <span class="brand-ripple brand-ripple-first" aria-hidden="true"></span>
                        <span class="brand-ripple brand-ripple-last" aria-hidden="true"></span>
                    </Show>
                </button>
            }.into_any()
        } else {
            view! {
                <A href="/" exact=true attr:class="brand"
                    attr:aria-label=move || state.locale.get().select("MCB / LOG，返回首页", "MCB / LOG, return home")>
                    <span node_ref=state.mark class="brand-mark" aria-hidden="true">"M"<span class="brand-mark-dot">"."</span></span>
                    <span class="brand-name" aria-hidden="true">"MCB"<span class="brand-name-muted">" / LOG"</span></span>
                </A>
            }.into_any()
        }
    }
}

fn render_brand_collapse_control(state: BrandState) -> impl IntoView {
    let label = move || state.locale.get().select("折叠侧栏", "Collapse sidebar");
    view! {
        <Show when=move || state.is_desktop && !state.preferences.get().is_sidebar_collapsed>
            <button node_ref=state.collapse class="sidebar-collapse topbar-button" type="button"
                disabled=move || !state.is_hydrated.get()
                aria-controls="site-sidebar" aria-expanded="true"
                aria-label=label title=label
                on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    let has_flight = should_animate_brand() && capture_brand_flight(state.brand, state.collapse, state.mark);
                    #[cfg(not(feature = "hydrate"))]
                    let has_flight = false;
                    state.is_collapsing.set(has_flight);
                    state.preferences.update(|value| value.is_sidebar_collapsed = true);
                    save_preferences(state.preferences.get_untracked());
                    #[cfg(feature = "hydrate")]
                    schedule_brand_focus(true, state.pending_focus);
                }
            ><Icon kind=IconKind::Sidebar/></button>
        </Show>
    }
}

#[component]
pub(super) fn SidebarBrand(is_hydrated: RwSignal<bool>, is_desktop: bool) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let state = BrandState {
        is_hydrated,
        is_desktop,
        preferences,
        locale,
        is_collapsing: RwSignal::new(false),
        brand: NodeRef::<html::Div>::new(),
        collapse: NodeRef::<html::Button>::new(),
        mark: NodeRef::<html::Span>::new(),
        #[cfg(feature = "hydrate")]
        pending_focus: StoredValue::new_local(RefCell::new(None::<AnimationFrameRequestHandle>)),
    };
    #[cfg(feature = "hydrate")]
    on_cleanup(move || cancel_brand_focus(state.pending_focus));
    Effect::new(move |_| {
        if !state.preferences.get().is_sidebar_collapsed && state.is_collapsing.get_untracked() {
            state.is_collapsing.set(false);
        }
    });
    view! {
        <div node_ref=state.brand class="sidebar-brand"
            class:is-collapsed=move || state.is_desktop && state.preferences.get().is_sidebar_collapsed
            class:is-collapsing=move || state.is_collapsing.get()
            on:animationend=move |event| {
                if event.animation_name() == "brand-ripple-last" {
                    state.is_collapsing.set(false);
                }
            }
            on:animationcancel=move |event| {
                if event.animation_name() == "brand-ripple-last" {
                    state.is_collapsing.set(false);
                }
            }>
            {render_brand_logo(state)}
            {render_brand_collapse_control(state)}
            <Show when=move || state.is_collapsing.get()>
                <span class="brand-flight" aria-hidden="true"><Icon kind=IconKind::Sidebar/></span>
            </Show>
        </div>
    }
}

#[component]
pub(super) fn SidebarNavigation(show_writing_link: bool) -> impl IntoView {
    let location = use_location();
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let home_active = move || location.pathname.get() == "/";
    let writing_active = move || {
        let pathname = location.pathname.get();
        pathname == "/writing" || pathname.starts_with("/writing/")
    };
    let focus_active = move || location.pathname.get() == "/focus";
    let about_active = move || location.pathname.get() == "/about";
    view! {
        <nav class="sidebar-nav" aria-label=move || locale.get().select("主导航", "Main navigation")>
            <A href="/" exact=true attr:class=move || if home_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().home() attr:aria-label=move || locale.get().home()>
                <Icon kind=IconKind::Home/><span class="sidebar-link-label">{move || locale.get().home()}</span>
            </A>
            {show_writing_link.then(|| view! {
                <A href="/writing" attr:class=move || if writing_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().writing() attr:aria-label=move || locale.get().writing()>
                    <Icon kind=IconKind::Writing/><span class="sidebar-link-label">{move || locale.get().writing()}</span>
                </A>
            })}
            <A href="/focus" exact=true attr:class=move || if focus_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().focus() attr:aria-label=move || locale.get().focus()>
                <Icon kind=IconKind::Focus/><span class="sidebar-link-label">{move || locale.get().focus()}</span>
            </A>
            <A href="/about" exact=true attr:class=move || if about_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().about() attr:aria-label=move || locale.get().about()>
                <Icon kind=IconKind::About/><span class="sidebar-link-label">{move || locale.get().about()}</span>
            </A>
        </nav>
    }
}

// Keep the existing pointer-capture lifecycle and SSR node refs under one owner;
// retaining the resize handlers here intentionally exceeds the helper-function limit.
#[component]
pub(super) fn Sidebar(viewport_width: RwSignal<u16>, is_hydrated: RwSignal<bool>) -> impl IntoView {
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let drag = RwSignal::new(None::<DragState>);
    let handle = NodeRef::<html::Div>::new();
    let is_dragging = move || drag.get().is_some();
    let reading_request = RwSignal::new(0_u64);
    let is_visible = Signal::derive(move || {
        viewport_width.get() >= SIDEBAR_MOBILE_BREAKPOINT_PX
            && !preferences.get().is_sidebar_collapsed
    });
    let maximum = move || available_max(viewport_width.get());
    let current_width = move || preferences.get().sidebar_width_px.min(maximum());
    let finish = move |should_save: bool| {
        if let Some(state) = drag.get_untracked() {
            drag.set(None);
            if should_save {
                save_preferences(preferences.get_untracked());
            } else {
                preferences.update(|value| value.sidebar_width_px = state.initial_width);
            }
            #[cfg(feature = "hydrate")]
            if let Some(body) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.body())
            {
                let _ = body.class_list().remove_1("is-resizing");
            }
        }
    };
    view! {
        <aside id="site-sidebar" class="site-sidebar" class:is-collapsed=move || preferences.get().is_sidebar_collapsed>
            <SidebarBrand is_hydrated is_desktop=true/>
            <ReadingNavigation is_hydrated is_visible reading_request/>
            <div class="sidebar-icon-nav">
                <SidebarNavigation show_writing_link=false/>
                <div class="navigation-page-footer">
                    <A href="/writing" attr:class="sidebar-link" attr:data-sidebar-level=""
                        attr:aria-label=move || locale.get().select("文章 / 书籍", "Writing / Books")
                        attr:title=move || locale.get().select("文章 / 书籍", "Writing / Books")
                        on:click=move |event| {
                            if event.button() != 0 || event.ctrl_key() || event.meta_key() || event.shift_key() || event.alt_key() {
                                return;
                            }
                            preferences.update(|value| value.is_sidebar_collapsed = false);
                            save_preferences(preferences.get_untracked());
                            reading_request.update(|request| *request = request.wrapping_add(1));
                        }>
                        <Icon kind=IconKind::Writing/><span class="sidebar-link-label">{move || locale.get().select("文章 / 书籍", "Writing / Books")}</span>
                    </A>
                </div>
            </div>
            <div
                node_ref=handle
                class="sidebar-resizer"
                role="separator"
                tabindex="0"
                aria-label=move || locale.get().select("调整侧栏宽度", "Resize sidebar")
                aria-orientation="vertical"
                aria-controls="site-sidebar"
                aria-valuemin=SIDEBAR_MIN_PX
                aria-valuemax=maximum
                aria-valuenow=current_width
                aria-valuetext=move || format!("{} {}", current_width(), locale.get().select("像素", "pixels"))
                class:is-dragging=is_dragging
                on:pointerdown=move |event| {
                    #[cfg(feature = "hydrate")]
                    {
                        if event.button() != 0 || preferences.get_untracked().is_sidebar_collapsed {
                            return;
                        }
                        let Some(element) = handle.get_untracked() else {
                            return;
                        };
                        if element.set_pointer_capture(event.pointer_id()).is_err() {
                            return;
                        }
                        let _ = element.focus();
                        if let Some(body) = web_sys::window().and_then(|window| window.document())
                            .and_then(|document| document.body())
                        {
                            let _ = body.class_list().add_1("is-resizing");
                        }
                        drag.set(Some(DragState {
                            pointer_id: event.pointer_id(),
                            start_x: event.client_x(),
                            initial_width: preferences.get_untracked().sidebar_width_px,
                            initial_visible_width: current_width(),
                        }));
                        event.prevent_default();
                    }
                    #[cfg(not(feature = "hydrate"))]
                    let _ = event;
                }
                on:pointermove=move |event| {
                    if let Some(state) = drag.get_untracked().filter(|state| state.pointer_id == event.pointer_id()) {
                        let width = (i32::from(state.initial_visible_width) + event.client_x() - state.start_x)
                            .clamp(i32::from(SIDEBAR_MIN_PX), i32::from(available_max(viewport_width.get_untracked())));
                        preferences.update(|value| value.sidebar_width_px = width as u16);
                    }
                }
                on:pointerup=move |event| {
                    if drag.get_untracked().is_some_and(|state| state.pointer_id == event.pointer_id()) {
                        finish(true);
                        #[cfg(feature = "hydrate")]
                        if let Some(element) = handle.get_untracked() {
                            let _ = element.release_pointer_capture(event.pointer_id());
                        }
                    }
                }
                on:pointercancel=move |event| {
                    if drag.get_untracked().is_some_and(|state| state.pointer_id == event.pointer_id()) {
                        finish(false);
                    }
                }
                on:keydown=move |event| {
                    if let Some(state) = drag.get_untracked().filter(|_| event.key() == "Escape") {
                        #[cfg(not(feature = "hydrate"))]
                        let _ = state;
                        finish(false);
                        #[cfg(feature = "hydrate")]
                        if let Some(element) = handle.get_untracked() {
                            let _ = element.release_pointer_capture(state.pointer_id);
                        }
                        event.prevent_default();
                        return;
                    }
                    let current = current_width();
                    let next = match event.key().as_str() {
                        "ArrowLeft" => current.saturating_sub(16).max(SIDEBAR_MIN_PX),
                        "ArrowRight" => current.saturating_add(16).min(maximum()),
                        "Home" => SIDEBAR_MIN_PX,
                        "End" => maximum(),
                        _ => return,
                    };
                    event.prevent_default();
                    preferences.update(|value| value.sidebar_width_px = next);
                    save_preferences(preferences.get_untracked());
                }
            ></div>
        </aside>
    }
}
