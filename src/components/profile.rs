use crate::{locale::Locale, profile::ProfileProject};
use leptos::prelude::*;

#[component]
pub(crate) fn ProjectCards(projects: &'static [ProfileProject]) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <ul class="project-grid">
            {projects.iter().map(|project| view! {
                <li class="project-card">
                    <h4><a href=project.url target="_blank" rel="noopener noreferrer">{project.name}<span aria-hidden="true">" ↗"</span></a></h4>
                    <p>{move || locale.get().select(project.summary_zh, project.summary_en)}</p>
                    <span class="project-stack">{project.stack}</span>
                </li>
            }).collect_view()}
        </ul>
    }
}

#[component]
pub(crate) fn CodeFlowCard() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div class="community-card">
            <img class="community-logo" src="/images/codeflow-logo.png" width="640" height="640" decoding="async" loading="lazy"
                alt=move || locale.get().select("CodeFlow·代码灵动社团 Logo", "CodeFlow·代码灵动 club logo")/>
            <div>
                <p>{move || locale.get().select(
                    "我创办了 CodeFlow·代码灵动社团，把对代码的热情延伸到共同学习与交流。",
                    "I founded CodeFlow·代码灵动, extending my enthusiasm for code into shared learning and exchange.")}</p>
                <p class="community-note">{move || locale.get().select(
                    "B站上有社团相关内容。", "Related CodeFlow content is available on Bilibili.")}</p>
            </div>
        </div>
    }
}
