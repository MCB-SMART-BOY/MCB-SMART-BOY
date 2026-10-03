use crate::content::POSTS;
use crate::markdown::render_markdown;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::{components::A, hooks::use_params_map};

use super::not_found::NotFoundPage;

#[component]
pub fn ArticlePage() -> impl IntoView {
    let params = use_params_map();

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
                    <A href="/writing" attr:class="article-back">"← 返回全部文章"</A>
                    <header class="article-header">
                        <div class="article-meta">
                            <span class="square" aria-hidden="true"></span>
                            <span>"DEMO / "{post.category}</span>
                            <span class="meta-divider">"/"</span>
                            <time datetime=post.iso_date>{post.date}</time>
                            <span class="meta-divider">"/"</span>
                            <span>{post.reading_time}</span>
                        </div>
                        <h1>{post.title}</h1>
                        <p>{post.summary}</p>
                    </header>
                    <div class="article-body prose" inner_html=render_markdown(post.body)></div>
                    <footer class="article-footer">
                        <span>"END OF NOTE "<span aria-hidden="true">"✳"</span></span>
                        <A href="/writing">"继续阅读 "<span aria-hidden="true">"↗"</span></A>
                    </footer>
                </article>
            }
            .into_any(),
            None => view! { <NotFoundPage/> }.into_any(),
        }
    }
}
