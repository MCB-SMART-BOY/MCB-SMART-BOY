use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    SsrMode, StaticSegment,
    components::{Route, Router, Routes},
    path,
};

#[cfg(feature = "ssr")]
use crate::preferences::parse_preferences;
use crate::{
    components::SiteShell,
    pages::{AboutPage, ArticlePage, FocusPage, HomePage, NotFoundPage, WritingPage},
    preferences::UiPreferences,
};
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;

#[cfg(feature = "ssr")]
fn read_cookie_preferences() -> UiPreferences {
    use axum::http::{header::COOKIE, request::Parts};
    use leptos::prelude::use_context;

    let cookie = use_context::<Parts>().and_then(|parts| {
        parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    });
    let value = cookie.as_deref().and_then(|header| {
        header
            .split(';')
            .filter_map(|part| part.trim().split_once('='))
            .find_map(|(name, value)| (name == "mcb-ui").then_some(value))
    });
    parse_preferences(value)
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let preferences = read_cookie_preferences();
    provide_context(preferences);
    let initial_theme = preferences.theme.as_str();
    let initial_ui = preferences.cookie_value();
    let initial_width = format!("--sidebar-width: {}px", preferences.sidebar_width_px);
    let theme_color = match initial_theme {
        "dark" => "#09070f",
        _ => "#f7faff",
    };
    view! {
        <!DOCTYPE html>
        <html lang="zh-CN" data-theme=initial_theme data-ui=initial_ui style=initial_width>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content=theme_color/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
                <link rel="stylesheet" href="/pkg/mcb-smart-boy.css"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body><App/></body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let preferences = use_context::<UiPreferences>().unwrap_or_default();
    provide_context(RwSignal::new(preferences));
    view! {
        <Router>
            <SiteShell>
                <Routes fallback=NotFoundPage>
                    <Route path=StaticSegment("") view=HomePage ssr=SsrMode::Async/>
                    <Route path=path!("/writing") view=WritingPage ssr=SsrMode::Async/>
                    <Route path=path!("/writing/:slug") view=ArticlePage ssr=SsrMode::Async/>
                    <Route path=path!("/focus") view=FocusPage ssr=SsrMode::Async/>
                    <Route path=path!("/about") view=AboutPage ssr=SsrMode::Async/>
                </Routes>
            </SiteShell>
        </Router>
    }
}
