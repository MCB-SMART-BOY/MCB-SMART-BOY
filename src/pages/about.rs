use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <Title text="关于 — MCB / LOG"/>
        <Meta name="description" content="了解 MCB-SMART-BOY 的关注方向以及这个 Rust 博客 demo。"/>
        <section class="page-intro about-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" ABOUT / THE PERSON BEHIND THE LOG"</span>
            <h1 id="page-title">"你好，"<span class="accent">"我是 MCB."</span></h1>
            <p>"一个关注系统编程、实时推理和安全的开发者。这个小站用来存放探索过程，而不只是最终答案。"</p>
        </section>
        <section class="about-layout">
            <div class="about-label">"A FEW THINGS"<br/>"ABOUT ME "<span aria-hidden="true">"↘"</span></div>
            <div class="about-text">
                <h2>"保持好奇，"<br/>"让想法落地。"</h2>
                <p>"项目原有介绍列出的方向包括 Rust / Python、Linux / NixOS、实时推理、系统编程、内核开发和计算机视觉。这里沿用这些信息，不代替个人履历，也不展示尚未提供的项目成果。"</p>
                <p>"本站目前是可运行的博客演示。文章是示例内容；等真实的想法和作品准备好后，可以直接替换。"</p>
                <a class="inline-link" href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">
                    "访问我的 GitHub "<span aria-hidden="true">"↗"</span>
                </a>
            </div>
        </section>
        <div class="back-link"><A href="/">"← 返回首页"</A></div>
    }
}
