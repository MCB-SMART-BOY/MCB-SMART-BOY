use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[component]
pub fn FocusPage() -> impl IntoView {
    view! {
        <Title text="关注领域 — MCB / LOG"/>
        <Meta name="description" content="系统编程、智能与安全，以及公开记录：MCB-SMART-BOY 持续探索的方向。"/>
        <section class="page-intro focus-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" 02 / CURRENT SIGNAL"</span>
            <h1 id="page-title">"持续关注"<span class="accent">"."</span></h1>
            <p>"不是完成清单，而是持续探索的问题空间。"</p>
        </section>
        <section class="section focus-section" aria-labelledby="focus-cards-title">
            <h2 class="sr-only" id="focus-cards-title">"关注方向"</h2>
            <div class="section-top"><span class="section-kicker">"WHERE CURIOSITY GOES"</span></div>
            <div class="focus-grid">
                <div class="focus-card">
                    <span class="focus-number">"[ 01 ]"</span><span class="focus-symbol" aria-hidden="true">"⌘"</span>
                    <h3>"系统编程"<span>" / SYSTEMS"</span></h3>
                    <p>"从语言到内核，追问抽象下面真正发生的事。"</p>
                    <span class="focus-foot">"RUST · LINUX · KERNEL"</span>
                </div>
                <div class="focus-card">
                    <span class="focus-number">"[ 02 ]"</span><span class="focus-symbol" aria-hidden="true">"✳"</span>
                    <h3>"智能与安全"<span>" / INTELLIGENCE"</span></h3>
                    <p>"关注实时推理、计算机视觉与可靠的软件边界。"</p>
                    <span class="focus-foot">"AI · SECURITY · VISION"</span>
                </div>
                <div class="focus-card">
                    <span class="focus-number">"[ 03 ]"</span><span class="focus-symbol" aria-hidden="true">"↗"</span>
                    <h3>"公开记录"<span>" / NOTES"</span></h3>
                    <p>"把实验过程写下来，让下一次探索有迹可循。"</p>
                    <span class="focus-foot">"BUILD · LEARN · SHARE"</span>
                </div>
            </div>
        </section>
    }
}
