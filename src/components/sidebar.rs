use leptos::{html, prelude::*};
use leptos_router::{components::A, hooks::use_location};

#[cfg(feature = "hydrate")]
use crate::preferences::persist_preferences;
use crate::preferences::{SIDEBAR_CONTENT_MIN_PX, SIDEBAR_MAX_PX, SIDEBAR_MIN_PX, UiPreferences};

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
pub(super) fn SidebarNavigation(
    #[prop(optional)] mobile: bool,
    #[prop(optional)] fallback: bool,
) -> impl IntoView {
    let location = use_location();
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let is_collapsed = move || !mobile && preferences.get().is_sidebar_collapsed;
    let home_active = move || location.pathname.get() == "/";
    let writing_active = move || {
        let pathname = location.pathname.get();
        pathname == "/writing" || pathname.starts_with("/writing/")
    };
    let focus_active = move || location.pathname.get() == "/focus";
    let about_active = move || location.pathname.get() == "/about";
    view! {
        <Show when=move || !fallback>
            <div class="sidebar-brand">
                <A href="/" exact=true attr:class="brand" attr:aria-label="MCB / LOG，返回首页">
                    <span class="brand-mark" aria-hidden="true">"M"<span class="brand-mark-dot">"."</span></span>
                    <span class="brand-name" aria-hidden="true">"MCB"<span class="brand-name-muted">" / LOG"</span></span>
                </A>
            </div>
        </Show>
        <nav class="sidebar-nav" aria-label="主导航">
            <A href="/" exact=true attr:class=move || if home_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title="首页" attr:aria-label="首页">
                <Icon kind=IconKind::Home/><span class="sidebar-link-label">"首页"</span>
            </A>
            <A href="/writing" attr:class=move || if writing_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title="文章" attr:aria-label="文章">
                <Icon kind=IconKind::Writing/><span class="sidebar-link-label">"文章"</span>
            </A>
            <A href="/focus" exact=true attr:class=move || if focus_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title="关注领域" attr:aria-label="关注领域">
                <Icon kind=IconKind::Focus/><span class="sidebar-link-label">"关注领域"</span>
            </A>
            <A href="/about" exact=true attr:class=move || if about_active() { "sidebar-link is-active" } else { "sidebar-link" } attr:title="关于" attr:aria-label="关于">
                <Icon kind=IconKind::About/><span class="sidebar-link-label">"关于"</span>
            </A>
        </nav>
        <span class="sr-only" aria-live="polite">{move || if is_collapsed() { "侧栏已折叠" } else { "侧栏已展开" }}</span>
    }
}

#[component]
pub(super) fn Sidebar(viewport_width: RwSignal<u16>) -> impl IntoView {
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
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
            <SidebarNavigation/>
            <div
                node_ref=handle
                class="sidebar-resizer"
                role="separator"
                tabindex="0"
                aria-label="调整侧栏宽度"
                aria-orientation="vertical"
                aria-controls="site-sidebar"
                aria-valuemin=SIDEBAR_MIN_PX
                aria-valuemax=maximum
                aria-valuenow=current_width
                aria-valuetext=move || format!("{} 像素", current_width())
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
