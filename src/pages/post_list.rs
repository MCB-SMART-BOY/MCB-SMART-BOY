use crate::{content::POSTS, locale::Locale};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub(super) fn PostList() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <ul class="post-list">
            {POSTS.iter().map(|post| view! {
                <li>
                    <A href=format!("/writing/{}", post.slug) attr:class="post-row">
                        <span class="post-index">{post.index}<span class="post-index-line" aria-hidden="true"></span></span>
                        <div class="post-content">
                            <span class="post-category">{post.category}" / "{move || locale.get().select("中文原文", "Chinese original")}</span>
                            <h3 class="post-title" lang="zh-CN">{post.title}</h3>
                            <span class="post-summary" lang="zh-CN">{post.summary}</span>
                        </div>
                        <span class="post-end"><time datetime=post.iso_date>{post.date}</time><span class="post-arrow" aria-hidden="true">"↗"</span></span>
                    </A>
                </li>
            }).collect_view()}
        </ul>
    }
}
