use leptos::{html, prelude::*};
use leptos_router::{components::A, hooks::use_location};

#[cfg(feature = "hydrate")]
use crate::preferences::persist_preferences;
use crate::{
    locale::Locale,
    preferences::{SIDEBAR_CONTENT_MIN_PX, SIDEBAR_MAX_PX, SIDEBAR_MIN_PX, UiPreferences},
};

use super::icons::{Icon, IconKind};

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

#[component]
pub(super) fn SidebarBrand(is_hydrated: RwSignal<bool>, is_desktop: bool) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let label = move || {
        if preferences.get().is_sidebar_collapsed {
            locale.get().select("展开侧栏", "Expand sidebar")
        } else {
            locale.get().select("折叠侧栏", "Collapse sidebar")
        }
    };
    view! {
        <div class="sidebar-brand">
            <A href="/" exact=true attr:class="brand" attr:aria-label=move || locale.get().select("MCB / LOG，返回首页", "MCB / LOG, return home")>
                <span class="brand-mark" aria-hidden="true">"M"<span class="brand-mark-dot">"."</span></span>
                <span class="brand-name" aria-hidden="true">"MCB"<span class="brand-name-muted">" / LOG"</span></span>
            </A>
            <Show when=move || is_desktop>
                <button
                    class="sidebar-collapse topbar-button"
                    type="button"
                    disabled=move || !is_hydrated.get()
                    aria-controls="site-sidebar"
                    aria-expanded=move || if preferences.get().is_sidebar_collapsed { "false" } else { "true" }
                    aria-label=label
                    title=label
                    on:click=move |_| {
                        preferences.update(|value| value.is_sidebar_collapsed = !value.is_sidebar_collapsed);
                        save_preferences(preferences.get_untracked());
                    }
                ><Icon kind=IconKind::Sidebar/></button>
            </Show>
        </div>
    }
}

#[component]
pub(super) fn SidebarNavigation() -> impl IntoView {
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
            <A href="/writing" attr:class=move || if writing_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().writing() attr:aria-label=move || locale.get().writing()>
                <Icon kind=IconKind::Writing/><span class="sidebar-link-label">{move || locale.get().writing()}</span>
            </A>
            <A href="/focus" exact=true attr:class=move || if focus_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().focus() attr:aria-label=move || locale.get().focus()>
                <Icon kind=IconKind::Focus/><span class="sidebar-link-label">{move || locale.get().focus()}</span>
            </A>
            <A href="/about" exact=true attr:class=move || if about_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title=move || locale.get().about() attr:aria-label=move || locale.get().about()>
                <Icon kind=IconKind::About/><span class="sidebar-link-label">{move || locale.get().about()}</span>
            </A>
        </nav>
    }
}

#[component]
pub(super) fn Sidebar(viewport_width: RwSignal<u16>, is_hydrated: RwSignal<bool>) -> impl IntoView {
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let drag = RwSignal::new(None::<DragState>);
    let handle = NodeRef::<html::Div>::new();
    let is_dragging = move || drag.get().is_some();
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
            <SidebarNavigation/>
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
