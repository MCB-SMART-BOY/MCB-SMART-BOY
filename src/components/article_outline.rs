use leptos::prelude::*;

use crate::{content::Post, locale::Locale, markdown::HeadingEntry};

#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_location;

#[cfg(feature = "hydrate")]
fn synchronize_location_hash(current_hash: RwSignal<Option<String>>) {
    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(
            &"failed to read article outline hash: browser window unavailable".into(),
        );
        current_hash.set(None);
        return;
    };
    match window.location().hash() {
        Ok(hash) => current_hash.set((!hash.is_empty()).then_some(hash)),
        Err(error) => {
            web_sys::console::error_2(
                &"failed to read article outline hash from location".into(),
                &error,
            );
            current_hash.set(None);
        }
    }
}

fn track_outline_hash() -> RwSignal<Option<String>> {
    let current_hash = RwSignal::new(None::<String>);
    #[cfg(feature = "hydrate")]
    {
        let location = use_location();
        Effect::new(move |_| {
            let _ = location.hash.get();
            synchronize_location_hash(current_hash);
        });
        let hash_listener = window_event_listener(leptos::ev::hashchange, move |_| {
            synchronize_location_hash(current_hash);
        });
        let pop_listener = window_event_listener(leptos::ev::popstate, move |_| {
            synchronize_location_hash(current_hash);
        });
        on_cleanup(move || {
            hash_listener.remove();
            pop_listener.remove();
        });
    }
    current_hash
}

#[component]
fn OutlineLinks(
    headings: &'static [HeadingEntry],
    min_level: u8,
    current_hash: RwSignal<Option<String>>,
    post: Option<&'static Post>,
) -> impl IntoView {
    headings
        .iter()
        .map(|heading| {
            let current_href = format!("#{}", heading.id);
            let href = match post {
                Some(post) => format!("/writing/{}{}", post.slug, current_href),
                None => current_href.clone(),
            };
            let depth = heading.level.saturating_sub(min_level);
            let text = heading.text.as_str();
            view! {
                <a
                    class="article-outline-link"
                    href=href
                    style=format!("--outline-depth: {depth}")
                    aria-current=move || (current_hash.get().as_deref() == Some(current_href.as_str()))
                        .then_some("location")
                    lang="zh-CN"
                >{text}</a>
            }
        })
        .collect_view()
}

#[component]
fn OutlineContent(
    headings: &'static [HeadingEntry],
    min_level: u8,
    current_hash: RwSignal<Option<String>>,
    locale: RwSignal<Locale>,
    post: Option<&'static Post>,
) -> impl IntoView {
    if headings.is_empty() {
        view! { <p class="article-outline-empty">{move || locale.get().select("本文暂无小标题", "No section headings")}</p> }.into_any()
    } else {
        view! {
            <div class="article-outline-links">
                <OutlineLinks headings min_level current_hash post/>
            </div>
        }
        .into_any()
    }
}

#[component]
pub(crate) fn ArticleOutline(headings: &'static [HeadingEntry]) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let current_hash = track_outline_hash();
    let min_level = headings
        .iter()
        .map(|heading| heading.level)
        .min()
        .unwrap_or(1);

    view! {
        <details class="article-outline-compact">
            <summary>{move || locale.get().select("本文目录", "On this page")}</summary>
            <OutlineContent headings min_level current_hash locale post=None/>
        </details>
    }
}

#[component]
pub(crate) fn SidebarOutline(post: &'static Post) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let headings = post.render().headings.as_slice();
    let min_level = headings
        .iter()
        .map(|heading| heading.level)
        .min()
        .unwrap_or(1);
    let current_hash = track_outline_hash();
    view! {
        <nav class="reading-navigation-paragraphs" aria-label=move || locale.get().select("段落目录", "Article sections")>
            <OutlineContent headings min_level current_hash locale post=Some(post)/>
        </nav>
    }
}
