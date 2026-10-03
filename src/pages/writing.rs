use crate::content::POSTS;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

use super::post_list::PostList;

#[component]
pub fn WritingPage() -> impl IntoView {
    view! {
        <Title text="文章 — MCB / LOG"/>
        <Meta name="description" content="浏览 MCB-SMART-BOY 博客 demo 的示例文章与实验笔记。"/>
        <section class="page-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" INDEX / 001"</span>
            <h1 id="page-title">"所有"<span class="accent">"记录."</span></h1>
            <p>"从一个问题开始，写下探索过程。以下均为 demo 示例文章，并非作者已发表作品。"</p>
        </section>
        <section class="section archive-section" aria-labelledby="archive-title">
            <h2 class="sr-only" id="archive-title">"2026 年示例文章"</h2>
            <div class="section-top">
                <span class="section-kicker">"THE ARCHIVE / 2026"</span>
                <span class="section-aside">{format!("{} ENTRIES", POSTS.len())}</span>
            </div>
            <PostList/>
        </section>
        <div class="back-link"><A href="/">"← 返回首页"</A></div>
    }
}
