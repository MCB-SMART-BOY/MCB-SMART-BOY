use leptos::{html, prelude::*};

use crate::{
    locale::Locale,
    preferences::{SIDEBAR_MOBILE_BREAKPOINT_PX, UiPreferences},
};
use leptos_router::hooks::use_location;

use super::{
    article_outline::provide_article_outline_context,
    current_directory::{CurrentDirectory, has_article_directory},
    icons::{Icon, IconKind},
    reading_navigation::ReadingNavigation,
    sidebar::{Sidebar, SidebarBrand, SidebarNavigation},
    topbar::Topbar,
};

#[cfg(feature = "hydrate")]
use super::reading_navigation::is_unmodified_click;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
fn focus_element(element: Option<web_sys::HtmlElement>) {
    if let Some(element) = element {
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        if let Err(source) = element.focus_with_options(&options) {
            web_sys::console::error_2(
                &"Failed to focus sidebar navigation destination".into(),
                &source,
            );
        }
    }
}
#[cfg(feature = "hydrate")]
fn focus_main_after_sidebar_navigation() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let has_navigation_focus = document
        .active_element()
        .is_some_and(|element| is_main_navigation_link(&element));
    if !has_navigation_focus {
        return;
    }
    let main = document
        .get_element_by_id("main")
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    focus_element(main);
}

#[cfg(feature = "hydrate")]
fn is_main_navigation_link(element: &web_sys::Element) -> bool {
    const SELECTOR: &str =
        ".site-sidebar a[href]:not([data-sidebar-level]):not([data-reading-branch])";
    let link = match element.closest(SELECTOR) {
        Ok(Some(link)) => link,
        Ok(None) => return false,
        Err(source) => {
            web_sys::console::error_2(&"Failed to find content navigation link".into(), &source);
            return false;
        }
    };
    link.get_attribute("href")
        .is_some_and(|href| href.starts_with('/'))
}

#[cfg(feature = "hydrate")]
fn sync_document_preferences(preferences: UiPreferences) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("data-theme", preferences.theme.as_str());
        let _ = root.set_attribute("data-ui", &preferences.cookie_value());
    }
    if let Ok(Some(meta)) = document.query_selector("meta[name=theme-color]") {
        let _ = meta.set_attribute("content", preferences.theme.color());
    }
}
#[cfg(feature = "hydrate")]
fn sync_document_locale(locale: Locale) {
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        if root.set_attribute("lang", locale.as_str()).is_err() {
            web_sys::console::error_1(&"Failed to update document language".into());
        }
    }
}

// Keep the shell's SSR tree and drawer lifecycle under one reactive owner;
// this component intentionally exceeds the helper-function length limit.
#[component]
pub fn SiteShell(children: Children) -> impl IntoView {
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let is_hydrated = RwSignal::new(false);
    let viewport_width = RwSignal::new(1440_u16);
    let is_drawer_open = RwSignal::new(false);
    let focus_main_after_close = RwSignal::new(false);
    let drawer = NodeRef::<html::Dialog>::new();
    let trigger = NodeRef::<html::Button>::new();
    let mobile_reading_request = RwSignal::new(0_u64);
    let workspace = NodeRef::<html::Div>::new();
    let pathname = use_location().pathname;
    provide_article_outline_context(pathname);
    let is_mobile_visible = Signal::derive(move || {
        viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX && is_drawer_open.get()
    });

    #[cfg(feature = "hydrate")]
    {
        super::directory_focus::watch_directory_focus(workspace, pathname);
        let location = use_location();
        let initial_path = location.pathname.get_untracked();
        Effect::new(move |previous_path: Option<String>| {
            let path = location.pathname.get();
            if previous_path.as_deref().unwrap_or(initial_path.as_str()) != path {
                focus_main_after_sidebar_navigation();
            }
            path
        });
        Effect::new(move || sync_document_preferences(preferences.get()));
        Effect::new(move || {
            let is_collapsed = preferences.get().is_sidebar_collapsed;
            let is_mobile = viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX;
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return;
            };
            let hidden_focus = if is_mobile {
                document.query_selector(".site-sidebar :focus")
            } else if is_collapsed {
                document.query_selector(".site-sidebar .reading-navigation :focus, .site-sidebar .sidebar-resizer:focus")
            } else {
                return;
            };
            if hidden_focus.ok().flatten().is_some() {
                let replacement = if is_mobile {
                    trigger.get_untracked().map(|element| element.into())
                } else {
                    document
                        .query_selector(".site-sidebar .brand-expand")
                        .ok()
                        .flatten()
                        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
                };
                focus_element(replacement);
            }
        });
        Effect::new(move || sync_document_locale(locale.get()));
        Effect::new(move || {
            if let Some(window) = web_sys::window() {
                if let Ok(width) = window.inner_width() {
                    viewport_width.set(
                        width
                            .as_f64()
                            .unwrap_or(f64::from(SIDEBAR_MOBILE_BREAKPOINT_PX))
                            .clamp(0.0, f64::from(u16::MAX)) as u16,
                    );
                }
                if let Some(root) = window
                    .document()
                    .and_then(|document| document.document_element())
                {
                    let _ = root.set_attribute("data-hydrated", "true");
                    is_hydrated.set(true);
                }
            }
        });
        let listener = window_event_listener(leptos::ev::resize, move |_| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let width = window
                .inner_width()
                .ok()
                .and_then(|width| width.as_f64())
                .unwrap_or(f64::from(SIDEBAR_MOBILE_BREAKPOINT_PX));
            let width = width.clamp(0.0, f64::from(u16::MAX)) as u16;
            viewport_width.set(width);
            if width >= SIDEBAR_MOBILE_BREAKPOINT_PX && is_drawer_open.get_untracked() {
                if let Some(dialog) = drawer.get_untracked() {
                    dialog.close();
                }
            }
        });
        on_cleanup(move || listener.remove());
    }

    view! {
        <a class="skip-link" href="#main">{move || locale.get().select("跳转到正文", "Skip to content")}</a>
        <div class="site-shell" style=move || format!("--sidebar-width: {}px", preferences.get().sidebar_width_px)
            on:click=move |event| {
                #[cfg(feature = "hydrate")]
                {
                    if !is_unmodified_click(&event) {
                        return;
                    }
                    let is_navigation_link = event.target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .is_some_and(|target| is_main_navigation_link(&target));
                    if is_navigation_link {
                        let main = web_sys::window().and_then(|window| window.document())
                            .and_then(|document| document.get_element_by_id("main"))
                            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
                        focus_element(main);
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = event;
            }>
            <Sidebar viewport_width is_hydrated/>
            <div node_ref=workspace class="site-workspace">
                <Topbar is_hydrated is_drawer_open drawer trigger/>
                <div class="mobile-fallback-nav">
                    <SidebarNavigation show_writing_link=true/>
                </div>
                <div class="content-layout" class:has-current-directory=move || has_article_directory(&pathname.get())>
                    <main id="main" class="site-content" tabindex="-1">{children()}</main>
                    <CurrentDirectory/>
                </div>
                <footer class="site-footer">
                    <div class="footer-top"><span class="footer-glyph" aria-hidden="true">"✳"</span><span>"KEEP BUILDING"<br/>"KEEP QUESTIONING."</span></div>
                    <div class="footer-bottom"><span>"© 2026 MCB-SMART-BOY "<span class="footer-separator">"/"</span>" BUILT WITH RUST"</span><span><a href="/writing">{move || locale.get().writing()}</a><a href="/about">{move || locale.get().about()}</a><a href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">"GITHUB ↗"</a></span></div>
                </footer>
            </div>
        </div>
        <dialog
            node_ref=drawer
            id="mobile-drawer"
            class="mobile-drawer"
            aria-label=move || locale.get().select("主导航", "Main navigation")
            on:close=move |_| {
                is_drawer_open.set(false);
                #[cfg(feature = "hydrate")]
                {
                    if focus_main_after_close.get_untracked() || viewport_width.get_untracked() >= SIDEBAR_MOBILE_BREAKPOINT_PX {
                        let main = web_sys::window().and_then(|window| window.document())
                            .and_then(|document| document.get_element_by_id("main"))
                            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
                        focus_element(main);
                    } else {
                        focus_element(trigger.get_untracked().map(|element| element.into()));
                    }
                }
                focus_main_after_close.set(false);
            }
            on:click=move |event| {
                #[cfg(feature = "hydrate")]
                {
                    if event.target().is_some_and(|target| target.dyn_into::<web_sys::HtmlDialogElement>().is_ok()) {
                        if let Some(dialog) = drawer.get_untracked() { dialog.close(); }
                        return;
                    }
                    if !is_unmodified_click(&event) {
                        return;
                    }
                    let is_internal_link = event.target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .and_then(|target| target.closest("a[href]:not([data-sidebar-level]):not([data-reading-branch])").ok().flatten())
                        .and_then(|link| link.get_attribute("href"))
                        .is_some_and(|href| href.starts_with('/'));
                    if is_internal_link {
                        focus_main_after_close.set(true);
                        if let Some(dialog) = drawer.get_untracked() { dialog.close(); }
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = event;
            }
        >
            <div class="drawer-panel">
                <button class="drawer-close topbar-button" type="button" aria-label=move || locale.get().select("关闭导航菜单", "Close navigation menu") on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    if let Some(dialog) = drawer.get_untracked() { dialog.close(); }
                }><Icon kind=IconKind::Close/></button>
                <SidebarBrand is_hydrated is_desktop=false/>
                <ReadingNavigation is_hydrated is_visible=is_mobile_visible reading_request=mobile_reading_request id_prefix="mobile"/>
            </div>
        </dialog>
    }
}
