use crate::{content::POSTS, locale::Locale, markdown::render_markdown};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::{components::A, hooks::use_params_map};

use super::not_found::NotFoundPage;

#[component]
pub fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));

    move || {
        let route_params = params.get();
        let post = route_params
            .get_str("slug")
            .and_then(|slug| POSTS.iter().find(|post| post.slug == slug));

        match post {
            Some(post) => view! {
                <Title text=format!("{} — MCB / LOG", post.title)/>
                <Meta name="description" content=post.summary/>
                <article class="article-page">
                    <A href="/writing" attr:class="article-back">{move || locale.get().select("← 返回全部文章", "← Back to all posts")}</A>
                    <header class="article-header">
                        <div class="article-meta">
                            <span class="square" aria-hidden="true"></span>
                            <span>{move || locale.get().select("示例文章 / ", "DEMO ARTICLE / ")}{post.category}</span>
                            <span class="meta-divider">"/"</span>
                            <time datetime=post.iso_date>{post.date}</time>
                            <span class="meta-divider">"/"</span>
                            <span>{move || match locale.get() {
                                Locale::ZhCn => format!("{} 分钟阅读", post.reading_minutes),
                                Locale::En => format!("{} min read", post.reading_minutes),
                            }}</span>
                        </div>
                        <span class="post-language">{move || locale.get().select("中文原文", "Chinese original")}</span>
                        <h1 lang="zh-CN">{post.title}</h1>
                        <p lang="zh-CN">{post.summary}</p>
                    </header>
                    <div class="article-body prose" lang="zh-CN" inner_html=render_markdown(post.body)></div>
                    <footer class="article-footer">
                        <span>"END OF NOTE "<span aria-hidden="true">"✳"</span></span>
                        <A href="/writing">{move || locale.get().select("继续阅读 ", "Keep reading ")}<span aria-hidden="true">"↗"</span></A>
                    </footer>
                </article>
            }
            .into_any(),
            None => view! { <NotFoundPage/> }.into_any(),
        }
    }
}
