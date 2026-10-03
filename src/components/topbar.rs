use leptos::{html, prelude::*};
use leptos_router::hooks::use_location;

use crate::{
    content::POSTS,
    preferences::{SIDEBAR_MOBILE_BREAKPOINT_PX, Theme, UiPreferences},
};

use super::{
    icons::{Icon, IconKind},
    sidebar::save_preferences,
};

#[component]
pub(super) fn Topbar(
    is_hydrated: RwSignal<bool>,
    viewport_width: RwSignal<u16>,
    is_drawer_open: RwSignal<bool>,
    drawer: NodeRef<html::Dialog>,
    trigger: NodeRef<html::Button>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = &drawer;
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let location = use_location();
    let title = move || {
        let pathname = location.pathname.get();
        match pathname.as_str() {
            "/" => "首页",
            "/writing" => "文章",
            "/focus" => "关注领域",
            "/about" => "关于",
            _ => pathname
                .strip_prefix("/writing/")
                .and_then(|slug| POSTS.iter().find(|post| post.slug == slug))
                .map_or("页面未找到", |post| post.title),
        }
    };
    let is_dark = move || preferences.get().theme == Theme::Dark;
    view! {
        <header class="site-topbar">
            <div class="topbar-leading">
                <button
                    node_ref=trigger
                    class="topbar-button sidebar-toggle"
                    type="button"
                    disabled=move || !is_hydrated.get()
                    aria-controls=move || if viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX { "mobile-drawer" } else { "site-sidebar" }
                    aria-expanded=move || {
                        let is_expanded = if viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX {
                            is_drawer_open.get()
                        } else {
                            !preferences.get().is_sidebar_collapsed
                        };
                        if is_expanded { "true" } else { "false" }
                    }
                    aria-label=move || {
                        if viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX { "打开导航菜单" }
                        else if preferences.get().is_sidebar_collapsed { "展开侧栏" }
                        else { "折叠侧栏" }
                    }
                    title=move || {
                        if viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX { "打开导航菜单" }
                        else if preferences.get().is_sidebar_collapsed { "展开侧栏" }
                        else { "折叠侧栏" }
                    }
                    on:click=move |_| {
                        if viewport_width.get_untracked() < SIDEBAR_MOBILE_BREAKPOINT_PX {
                            #[cfg(feature = "hydrate")]
                            if let Some(dialog) = drawer.get_untracked() {
                                if dialog.show_modal().is_ok() {
                                    is_drawer_open.set(true);
                                } else {
                                    web_sys::console::error_1(&"无法打开导航抽屉".into());
                                }
                            }
                        } else {
                            preferences.update(|value| value.is_sidebar_collapsed = !value.is_sidebar_collapsed);
                            save_preferences(preferences.get_untracked());
                        }
                    }
                ><Icon kind=IconKind::Sidebar/></button>
                <span class="topbar-title">{title}</span>
            </div>
            <div class="topbar-actions">
                <button
                    class="topbar-button theme-toggle"
                    type="button"
                    disabled=move || !is_hydrated.get()
                    aria-label=move || if is_dark() { "切换到日间模式" } else { "切换到夜间模式" }
                    title=move || if is_dark() { "切换到日间模式" } else { "切换到夜间模式" }
                    on:click=move |_| {
                        preferences.update(|value| value.theme = match value.theme {
                            Theme::Light => Theme::Dark,
                            Theme::Dark => Theme::Light,
                        });
                        save_preferences(preferences.get_untracked());
                    }
                >
                    {move || if is_dark() {
                        view! { <Icon kind=IconKind::Sun/> }.into_any()
                    } else {
                        view! { <Icon kind=IconKind::Moon/> }.into_any()
                    }}
                </button>
                <a class="topbar-github" href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer" aria-label="打开 MCB-SMART-BOY 的 GitHub 主页（新窗口）">
                    <span>"GITHUB"</span><Icon kind=IconKind::External/>
                </a>
            </div>
        </header>
    }
}
