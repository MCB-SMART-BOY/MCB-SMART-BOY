use crate::{content::POSTS, locale::Locale};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

use super::post_list::PostList;

#[component]
pub fn WritingPage() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <Title text=move || format!("{} — MCB / LOG", locale.get().writing())/>
        <Meta name="description" content=move || locale.get().select(
            "浏览 MCB-SMART-BOY 博客 demo 的示例文章与实验笔记。",
            "Browse demo articles and experiment notes on the MCB-SMART-BOY blog.",
        )/>
        <section class="page-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" INDEX / 001"</span>
            <h1 id="page-title">{move || locale.get().select("所有", "All ")}<span class="accent">{move || locale.get().select("记录.", "notes.")}</span></h1>
            <p>{move || locale.get().select("从一个问题开始，写下探索过程。以下均为 demo 示例文章，并非作者已发表作品。", "Start with a question and record the exploration. These are demo articles, not published work by the author.")}</p>
        </section>
        <section class="section archive-section" aria-labelledby="archive-title">
            <h2 class="sr-only" id="archive-title">{move || locale.get().select("2026 年示例文章", "Demo articles from 2026")}</h2>
            <div class="section-top">
                <span class="section-kicker">"THE ARCHIVE / 2026"</span>
                <span class="section-aside">{move || format!("{} {}", POSTS.len(), locale.get().select("篇文章", "ENTRIES"))}</span>
            </div>
            <PostList/>
        </section>
        <div class="back-link"><A href="/">{move || locale.get().select("← 返回首页", "← Back home")}</A></div>
    }
}
