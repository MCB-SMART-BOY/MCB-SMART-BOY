use crate::locale::Locale;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub fn AboutPage() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <Title text=move || format!("{} — MCB / LOG", locale.get().about())/>
        <Meta name="description" content=move || locale.get().select(
            "了解 MCB-SMART-BOY 的关注方向以及这个 Rust 博客 demo。",
            "Learn about MCB-SMART-BOY's interests and this Rust blog demo.",
        )/>
        <section class="page-intro about-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" ABOUT / THE PERSON BEHIND THE LOG"</span>
            <h1 id="page-title">{move || locale.get().select("你好，", "Hi, ")}<span class="accent">{move || locale.get().select("我是 MCB.", "I'm MCB.")}</span></h1>
            <p>{move || locale.get().select("一个关注系统编程、实时推理和安全的开发者。这个小站用来存放探索过程，而不只是最终答案。", "I'm a developer interested in systems programming, real-time inference, and security. This site is for the exploration process, not just final answers.")}</p>
        </section>
        <section class="about-layout">
            <div class="about-label">"A FEW THINGS"<br/>"ABOUT ME "<span aria-hidden="true">"↘"</span></div>
            <div class="about-text">
                <h2>{move || locale.get().select("保持好奇，", "Stay curious,")}<br/>{move || locale.get().select("让想法落地。", "put ideas into practice.")}</h2>
                <p>{move || locale.get().select("项目原有介绍列出的方向包括 Rust / Python、Linux / NixOS、实时推理、系统编程、内核开发和计算机视觉。这里沿用这些信息，不代替个人履历，也不展示尚未提供的项目成果。", "The existing project introduction lists Rust / Python, Linux / NixOS, real-time inference, systems programming, kernel development, and computer vision. This site draws on those interests; it is not a résumé and makes no claim about projects that have not been provided.")}</p>
                <p>{move || locale.get().select("本站目前是可运行的博客演示。文章是示例内容；等真实的想法和作品准备好后，可以直接替换。", "This site is currently a working blog demo. The articles are examples; replace them when real ideas and work are ready.")}</p>
                <a class="inline-link" href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">
                    {move || locale.get().select("访问我的 GitHub ", "Visit my GitHub ")}<span aria-hidden="true">"↗"</span>
                </a>
            </div>
        </section>
        <div class="back-link"><A href="/">{move || locale.get().select("← 返回首页", "← Back home")}</A></div>
    }
}
