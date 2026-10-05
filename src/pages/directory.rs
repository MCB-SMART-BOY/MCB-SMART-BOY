use crate::{
    content::{ContentEntry, Directory},
    locale::Locale,
};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub(super) fn DirectoryContent(directory: &'static Directory, is_root: bool) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let introduction = directory.render();
    let parent_path = directory.parent_path;
    view! {
        <Title text=format!("{} — MCB / LOG", directory.title)/>
        <Meta name="description" content=move || locale.get().select(
            "浏览按真实目录整理的文章与笔记。",
            "Browse articles and notes in their original directories.",
        )/>
        <article class=if is_root { "directory-page is-root" } else { "directory-page" }>
            {parent_path.map(|path| view! {
                <div class="back-link"><A href=path>{move || locale.get().select("← 返回上级目录", "← Back to parent directory")}</A></div>
            })}
            <header class="page-intro">
                <span class="section-kicker"><span class="square" aria-hidden="true"></span>" INDEX / 001"</span>
                <h1 lang="zh-CN">{directory.title}</h1>
                {(!introduction.html.is_empty()).then(|| view! {
                    <div class="directory-introduction prose" lang="zh-CN" inner_html=introduction.html.as_str()></div>
                })}
            </header>
            <section class="section archive-section" aria-labelledby="directory-entries-title">
                <h2 id="directory-entries-title">{move || locale.get().select("本级内容", "In this directory")}</h2>
                <ul class="directory-list">
                    {directory.entries.iter().map(|entry| match *entry {
                        ContentEntry::Directory(child) => view! {
                            <li class="directory-entry">
                                <h3 lang="zh-CN"><A href=child.path>{child.title}</A></h3>
                                <p class="directory-entry-summary">{move || match locale.get() {
                                    Locale::ZhCn => format!("{} 项内容", child.entries.len()),
                                    Locale::En => format!("Entries: {}", child.entries.len()),
                                }}</p>
                            </li>
                        }.into_any(),
                        ContentEntry::Article(post) => view! {
                            <li class="directory-entry">
                                <h3 lang="zh-CN"><A href=post.path>{post.title}</A></h3>
                                <p class="directory-entry-summary" lang="zh-CN">{post.summary}</p>
                            </li>
                        }.into_any(),
                    }).collect_view()}
                </ul>
                {directory.entries.is_empty().then(|| view! {
                    <p>{move || locale.get().select("暂无内容", "No content yet")}</p>
                })}
            </section>
            {is_root.then(|| view! {
                <div class="back-link"><A href="/">{move || locale.get().select("← 返回首页", "← Back home")}</A></div>
            })}
        </article>
    }
}
