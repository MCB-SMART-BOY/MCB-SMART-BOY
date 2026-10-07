use leptos::{html, prelude::*};

#[cfg(feature = "hydrate")]
use crate::locale::{LocalePersistError, persist_locale};
use crate::{
    locale::Locale,
    preferences::{Theme, UiPreferences},
};

use super::{
    breadcrumbs::Breadcrumbs,
    icons::{Icon, IconKind},
    share::ShareButton,
    sidebar::save_preferences,
};

#[component]
fn MobileDrawerButton(
    is_hydrated: RwSignal<bool>,
    is_drawer_open: RwSignal<bool>,
    drawer: NodeRef<html::Dialog>,
    trigger: NodeRef<html::Button>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = &drawer;
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <button node_ref=trigger class="topbar-button mobile-menu-toggle"
            type="button" disabled=move || !is_hydrated.get()
            aria-controls="mobile-drawer"
            aria-expanded=move || if is_drawer_open.get() { "true" } else { "false" }
            aria-label=move || locale.get().select("打开导航菜单", "Open navigation menu")
            title=move || locale.get().select("打开导航菜单", "Open navigation menu")
            on:click=move |_| {
                #[cfg(feature = "hydrate")]
                if let Some(dialog) = drawer.get_untracked() {
                    match dialog.show_modal() {
                        Ok(()) => is_drawer_open.set(true),
                        Err(source) => web_sys::console::error_2(
                            &"Failed to open navigation drawer".into(), &source,
                        ),
                    }
                }
            }
        ><Icon kind=IconKind::Sidebar/></button>
    }
}

#[component]
fn LanguageToggle(is_hydrated: RwSignal<bool>) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let has_save_error = RwSignal::new(false);
    view! {
        <div class="language-action">
            <button class="topbar-button topbar-language" type="button"
                disabled=move || !is_hydrated.get()
                aria-label=move || locale.get().select("切换到英文", "Switch to Chinese")
                title=move || locale.get().select("切换到英文", "Switch to Chinese")
                on:click=move |_| {
                    let next = if locale.get_untracked() == Locale::ZhCn {
                        Locale::En
                    } else {
                        Locale::ZhCn
                    };
                    locale.set(next);
                    #[cfg(feature = "hydrate")]
                    match persist_locale(next) {
                        Ok(()) => has_save_error.set(false),
                        Err(LocalePersistError::DocumentUnavailable) => {
                            web_sys::console::error_1(&"Failed to save language preference: document unavailable".into());
                            has_save_error.set(true);
                        }
                        Err(LocalePersistError::CookieWriteFailed(source)) => {
                            web_sys::console::error_2(&"Failed to save language preference: cookie write".into(), &source);
                            has_save_error.set(true);
                        }
                    }
                }
            >{move || locale.get().select("EN", "中")}</button>
            <span class="language-error" role="status" aria-live="polite">
                {move || if has_save_error.get() {
                    locale.get().select(
                        "语言已切换，但无法保存；刷新后可能恢复默认语言",
                        "Language changed, but could not be saved; reloading may restore the default",
                    )
                } else { "" }}
            </span>
        </div>
    }
}

#[component]
fn ThemeButton(is_hydrated: RwSignal<bool>) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let is_dark = move || preferences.get().theme == Theme::Dark;
    let label = move || {
        if is_dark() {
            locale
                .get()
                .select("切换到日间模式", "Switch to light theme")
        } else {
            locale
                .get()
                .select("切换到夜间模式", "Switch to dark theme")
        }
    };
    view! {
        <button class="topbar-button theme-toggle" type="button"
            disabled=move || !is_hydrated.get()
            aria-label=label title=label
            on:click=move |_| {
                preferences.update(|value| value.theme = match value.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                });
                save_preferences(preferences.get_untracked());
            }
        >
            {move || if is_dark() {
                view! { <Icon kind=IconKind::Moon/> }.into_any()
            } else {
                view! { <Icon kind=IconKind::Sun/> }.into_any()
            }}
        </button>
    }
}

#[component]
pub(super) fn Topbar(
    is_hydrated: RwSignal<bool>,
    is_drawer_open: RwSignal<bool>,
    drawer: NodeRef<html::Dialog>,
    trigger: NodeRef<html::Button>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <header class="site-topbar">
            <Breadcrumbs/>
            <MobileDrawerButton is_hydrated is_drawer_open drawer trigger/>
            <div class="topbar-actions">
                <LanguageToggle is_hydrated/>
                <ThemeButton is_hydrated/>
                <a class="topbar-github" href="https://github.com/MCB-SMART-BOY"
                    target="_blank" rel="noopener noreferrer"
                    aria-label=move || locale.get().select(
                        "打开 MCB-SMART-BOY 的 GitHub 主页（新窗口）",
                        "Open MCB-SMART-BOY on GitHub (new window)",
                    )
                    title="GitHub"
                ><Icon kind=IconKind::Github/></a>
                <a class="topbar-github topbar-bilibili" href="https://space.bilibili.com/3493283751791185"
                    target="_blank" rel="noopener noreferrer"
                    aria-label=move || locale.get().select(
                        "在新窗口打开 CodeFlow 社团 B站主页",
                        "Open CodeFlow club on Bilibili in a new window",
                    )
                    title=move || locale.get().select(
                        "CodeFlow 社团 B站主页（新窗口）",
                        "CodeFlow club on Bilibili (new window)",
                    )
                ><Icon kind=IconKind::Bilibili/></a>
                <ShareButton is_hydrated/>
            </div>
        </header>
    }
}
