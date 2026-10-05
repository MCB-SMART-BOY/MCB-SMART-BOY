use crate::{components::article_outline::ArticleOutline, content::Post, locale::Locale};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub(super) fn ArticlePage(post: &'static Post) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let rendered = post.render();
    view! {
        <Title text=format!("{} — MCB / LOG", post.title)/>
        <Meta name="description" content=post.summary/>
        <article class="article-page">
            <A href=post.parent_path attr:class="article-back">
                {move || locale.get().select("← 返回上级目录", "← Back to parent directory")}
            </A>
            <header class="article-header">
                <div class="article-meta">
                    <span class="square" aria-hidden="true"></span>
                    <span>{move || locale.get().select("文章", "ARTICLE")}{(!post.category.is_empty()).then_some(" / ")}{post.category}</span>
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
                <A href=post.parent_path>{move || locale.get().select("继续阅读 ", "Keep reading ")}<span aria-hidden="true">"↗"</span></A>
            </footer>
        </article>
    }
}
