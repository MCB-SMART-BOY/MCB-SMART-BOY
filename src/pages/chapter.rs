use crate::{
    content::find_post,
    locale::Locale,
    reading::{ReadingRoute, resolve_reading_route},
};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::{components::A, hooks::use_location};

use super::not_found::NotFoundPage;

#[component]
pub fn ChapterPage() -> impl IntoView {
    let location = use_location();
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));

    move || {
        match resolve_reading_route(&location.pathname.get()) {
        Some(ReadingRoute::Chapter(book, chapter)) => view! {
            <Title text=format!("{} — MCB / LOG", chapter.title)/>
            <Meta name="description" content=move || locale.get().select(
                "按章节浏览示例合集中的中文文章。",
                "Browse the original Chinese articles in this demo chapter.",
            )/>
            <article class="chapter-page">
                <div class="back-link">
                    <A href=format!("/writing/books/{}", book.slug)>
                        {move || locale.get().select("← 返回书籍", "← Back to book")}
                    </A>
                </div>
                <header class="page-intro">
                    <span class="section-kicker"><span class="square" aria-hidden="true"></span>" CHAPTER"</span>
                    <h1 lang="zh-CN">{chapter.title}</h1>
                    <p>{move || format!("{} {}", chapter.post_slugs.len(), locale.get().select("篇文章", "articles"))}</p>
                </header>
                <section class="section archive-section" aria-labelledby="chapter-articles-title">
                    <h2 id="chapter-articles-title">{move || locale.get().select("本章文章", "Articles in this chapter")}</h2>
                    {if chapter.post_slugs.is_empty() {
                        view! { <p>{move || locale.get().select("暂无文章", "No articles yet")}</p> }.into_any()
                    } else {
                        view! {
                            <ul class="chapter-articles">
                                {chapter.post_slugs.iter().map(|slug| match find_post(slug) {
                                    Some(post) => view! {
                                        <li class="chapter-article">
                                            <h3 lang="zh-CN"><A href=format!("/writing/{}", post.slug)>{post.title}</A></h3>
                                            <p lang="zh-CN">{post.summary}</p>
                                        </li>
                                    }.into_any(),
                                    None => view! { <li>{format!("Missing article reference: {slug}")}</li> }.into_any(),
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </section>
            </article>
        }.into_any(),
        _ => view! { <NotFoundPage/> }.into_any(),
    }
    }
}
