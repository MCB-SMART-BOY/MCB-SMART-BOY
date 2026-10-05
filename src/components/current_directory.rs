use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    locale::Locale,
    reading::{ReadingRoute, resolve_reading_route},
};

use super::article_outline::ArticleSectionLinks;

pub(super) fn has_article_directory(pathname: &str) -> bool {
    matches!(
        resolve_reading_route(pathname),
        Some(ReadingRoute::Article(post)) if !post.render().headings.is_empty()
    )
}

#[component]
pub(super) fn CurrentDirectory() -> impl IntoView {
    let pathname = use_location().pathname;
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));

    move || {
        let Some(ReadingRoute::Article(post)) = resolve_reading_route(&pathname.get()) else {
            return ().into_any();
        };
        if post.render().headings.is_empty() {
            return ().into_any();
        }
        view! {
            <aside class="current-directory" aria-label=move || locale.get().select("本文目录", "On this page")>
                <div class="current-directory-header">
                    <h2 tabindex="-1">{move || locale.get().select("本文目录", "On this page")}</h2>
                </div>
                <div class="current-directory-links"><ArticleSectionLinks post/></div>
            </aside>
        }
        .into_any()
    }
}
