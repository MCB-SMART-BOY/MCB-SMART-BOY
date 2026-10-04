use crate::{
    content::Book,
    locale::Locale,
    reading::{ReadingRoute, build_chapter_path, resolve_reading_route},
};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::{components::A, hooks::use_location};

use super::not_found::NotFoundPage;

#[component]
pub(super) fn BookIntroduction(book: &'static Book) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <p class="book-introduction" lang="zh-CN">{book.introduction}</p>
        <p>{move || format!("{} {} · {} {}",
            book.chapters.len(), locale.get().select("章", "chapters"),
            book.chapters.iter().map(|chapter| chapter.post_slugs.len()).sum::<usize>(),
            locale.get().select("篇文章", "articles"),
        )}</p>
    }
}

#[component]
pub(super) fn BookChapterLinks(book: &'static Book) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    if book.chapters.is_empty() {
        view! { <p>{move || locale.get().select("暂无章节", "No chapters yet")}</p> }.into_any()
    } else {
        view! {
            <ul class="book-chapters">
                {book.chapters.iter().map(|chapter| view! {
                    <li class="book-chapter">
                        <A href=build_chapter_path(book, chapter)><span lang="zh-CN">{chapter.title}</span></A>
                        <span>{move || format!("{} {}",
                            chapter.post_slugs.len(),
                            locale.get().select("篇文章", "articles"),
                        )}</span>
                    </li>
                }).collect_view()}
            </ul>
        }.into_any()
    }
}

#[component]
pub fn BookPage() -> impl IntoView {
    let location = use_location();
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));

    move || {
        let book = match resolve_reading_route(&location.pathname.get()) {
            Some(ReadingRoute::Book(book)) => Some(book),
            _ => None,
        };
        match book {
            Some(book) => view! {
                <Title text=format!("{} — MCB / LOG", book.title)/>
                <Meta name="description" content=move || locale.get().select(
                    "示例文章按主题整理的演示合集，并非连续教程。",
                    "A demonstration collection of sample articles by topic, not a sequential tutorial.",
                )/>
                <article class="book-page">
                    <div class="back-link"><A href="/writing">{move || locale.get().select("← 返回书籍", "← Back to books")}</A></div>
                    <header class="page-intro">
                        <span class="section-kicker"><span class="square" aria-hidden="true"></span>" DEMO COLLECTION"</span>
                        <h1 lang="zh-CN">{book.title}</h1>
                        <BookIntroduction book/>
                    </header>
                    <section class="section archive-section" aria-labelledby="book-chapters-title">
                        <h2 id="book-chapters-title">{move || locale.get().select("选择章节", "Choose a chapter")}</h2>
                        <BookChapterLinks book/>
                    </section>
                </article>
            }.into_any(),
            None => view! { <NotFoundPage/> }.into_any(),
        }
    }
}
