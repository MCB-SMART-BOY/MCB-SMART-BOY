use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    SsrMode,
    components::{Route, Router, Routes},
    path,
};

use crate::{
    components::SiteShell,
    locale::Locale,
    pages::{NotFoundPage, SiteContent},
    preferences::UiPreferences,
};
#[cfg(feature = "ssr")]
use crate::{
    locale::parse_locale,
    preferences::{find_cookie_value, parse_preferences},
};
#[cfg(feature = "ssr")]
use leptos_meta::{HashedStylesheet, MetaTags};

#[cfg(feature = "ssr")]
fn read_cookie_preferences() -> (UiPreferences, Locale) {
    use axum::http::{header::COOKIE, request::Parts};

    let cookie = use_context::<Parts>().and_then(|parts| {
        parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    });
    let value = cookie.as_deref().unwrap_or_default();
    (
        parse_preferences(find_cookie_value(value, "mcb-ui")),
        parse_locale(find_cookie_value(value, "mcb-lang")),
    )
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let (preferences, locale) = read_cookie_preferences();
    provide_context(preferences);
    provide_context(locale);
    let initial_theme = preferences.theme.as_str();
    let initial_ui = preferences.cookie_value();
    let initial_width = format!("--sidebar-width: {}px", preferences.sidebar_width_px);
    let theme_color = preferences.theme.color();
    view! {
        <!DOCTYPE html>
        <html lang=locale.as_str() data-theme=initial_theme data-ui=initial_ui.clone()
            data-ssr-ui=initial_ui data-ssr-lang=locale.as_str() style=initial_width>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content=theme_color/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
                <script inner_html=crate::preferences::THEME_BOOTSTRAP></script>
                <HashedStylesheet options=options.clone()/>
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
    let locale = use_context::<Locale>().unwrap_or_default();
    provide_context(RwSignal::new(locale));
    view! {
        <Router>
            <SiteShell>
                <Routes fallback=NotFoundPage>
                    <Route path=path!("/*path") view=SiteContent ssr=SsrMode::Async/>
                </Routes>
            </SiteShell>
        </Router>
    }
}
