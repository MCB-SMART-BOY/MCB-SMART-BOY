use crate::{
    components::article_outline::ArticleOutline,
    content::{find_post, find_post_location},
    locale::Locale,
    reading::build_chapter_path,
};
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
        let post = route_params.get_str("slug").and_then(find_post);

        match post {
            Some(post) => {
                let rendered = post.render();
                let chapter_location = find_post_location(post.slug);
                let parent_path = chapter_location
                    .map(|(book, chapter)| build_chapter_path(book, chapter))
                    .unwrap_or_else(|| "/writing".to_owned());
                let is_in_chapter = chapter_location.is_some();
                view! {
                    <Title text=format!("{} — MCB / LOG", post.title)/>
                    <Meta name="description" content=post.summary/>
                    <article class="article-page">
                        <A href=parent_path.clone() attr:class="article-back">{move || if is_in_chapter {
                            locale.get().select("← 返回章节", "← Back to chapter")
                        } else {
                            locale.get().select("← 返回书籍", "← Back to books")
                        }}</A>
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
                        <ArticleOutline headings=rendered.headings.as_slice()/>
                        <div class="article-body prose" lang="zh-CN" inner_html=rendered.html.as_str()></div>
                        <footer class="article-footer">
                            <span>"END OF NOTE "<span aria-hidden="true">"✳"</span></span>
                            <A href=parent_path>{move || locale.get().select("继续阅读 ", "Keep reading ")}<span aria-hidden="true">"↗"</span></A>
                        </footer>
                    </article>
                }
                .into_any()
            }
            None => view! { <NotFoundPage/> }.into_any(),
        }
    }
}
