use crate::{
    content::{BOOKS, POSTS, find_post_location},
    locale::Locale,
};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

use super::book::{BookChapterLinks, BookIntroduction};

#[component]
pub fn WritingPage() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <Title text=move || format!("{} — MCB / LOG", locale.get().writing())/>
        <Meta name="description" content=move || locale.get().select(
            "按书籍与章节浏览 MCB-SMART-BOY 博客的示例文章。",
            "Browse sample MCB-SMART-BOY articles by book and chapter.",
        )/>
        <section class="page-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" INDEX / 001"</span>
            <h1 id="page-title">{move || locale.get().select("书籍", "Books")}</h1>
            <p>{move || locale.get().select(
                "以下是按主题整理的演示合集，不代表文章原本是连续教程；文章正文仍为中文原文。",
                "This is a demonstration collection grouped by topic, not a sequential tutorial. The original articles remain in Chinese.",
            )}</p>
        </section>
        <section class="section archive-section" aria-labelledby="archive-title">
            <h2 id="archive-title">{move || locale.get().select("选择书籍", "Choose a book")}</h2>
            <ul class="book-list">
                {BOOKS.iter().map(|book| view! {
                    <li class="book-card">
                        <section aria-label=book.title>
                            <h3 lang="zh-CN"><A href=format!("/writing/books/{}", book.slug)>{book.title}</A></h3>
                            <BookIntroduction book/>
                            <BookChapterLinks book/>
                        </section>
                    </li>
                }).collect_view()}
            </ul>
            {(!POSTS.iter().all(|post| find_post_location(post.slug).is_some())).then(|| view! {
                <section class="standalone-posts" aria-labelledby="standalone-title">
                    <h2 id="standalone-title">{move || locale.get().select("独立文章", "Standalone articles")}</h2>
                    <ul class="post-list">
                        {POSTS.iter().filter(|post| find_post_location(post.slug).is_none()).map(|post| view! {
                            <li><A href=format!("/writing/{}", post.slug) attr:class="post-row"><span lang="zh-CN">{post.title}</span></A></li>
                        }).collect_view()}
                    </ul>
                </section>
            })}
        </section>
        <div class="back-link"><A href="/">{move || locale.get().select("← 返回首页", "← Back home")}</A></div>
    }
}
