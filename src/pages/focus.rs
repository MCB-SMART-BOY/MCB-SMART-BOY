use crate::locale::Locale;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[component]
pub fn FocusPage() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <Title text=move || format!("{} — MCB / LOG", locale.get().focus())/>
        <Meta name="description" content=move || locale.get().select(
            "系统编程、智能与安全，以及公开记录：MCB-SMART-BOY 持续探索的方向。",
            "Systems programming, intelligence and security, and public notes: areas MCB-SMART-BOY continues to explore.",
        )/>
        <section class="page-intro focus-intro" aria-labelledby="page-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" 02 / CURRENT SIGNAL"</span>
            <h1 id="page-title">{move || locale.get().select("持续关注", "Areas of focus")}<span class="accent">"."</span></h1>
            <p>{move || locale.get().select("不是完成清单，而是持续探索的问题空间。", "Not a list of achievements, but questions to keep exploring.")}</p>
        </section>
        <section class="section focus-section" aria-labelledby="focus-cards-title">
            <h2 class="sr-only" id="focus-cards-title">{move || locale.get().select("关注方向", "Areas of focus")}</h2>
            <div class="section-top"><span class="section-kicker">"WHERE CURIOSITY GOES"</span></div>
            <div class="focus-grid">
                <div class="focus-card">
                    <span class="focus-number">"[ 01 ]"</span><span class="focus-symbol" aria-hidden="true">"⌘"</span>
                    <h3>{move || locale.get().select("系统编程", "Systems programming")}<span>" / SYSTEMS"</span></h3>
                    <p>{move || locale.get().select("从语言到内核，追问抽象下面真正发生的事。", "From languages to kernels, asking what really happens beneath the abstractions.")}</p>
                    <span class="focus-foot">"RUST · LINUX · KERNEL"</span>
                </div>
                <div class="focus-card">
                    <span class="focus-number">"[ 02 ]"</span><span class="focus-symbol" aria-hidden="true">"✳"</span>
                    <h3>{move || locale.get().select("智能与安全", "Intelligence & security")}<span>" / INTELLIGENCE"</span></h3>
                    <p>{move || locale.get().select("关注实时推理、计算机视觉与可靠的软件边界。", "Exploring real-time inference, computer vision, and reliable software boundaries.")}</p>
                    <span class="focus-foot">"AI · SECURITY · VISION"</span>
                </div>
                <div class="focus-card">
                    <span class="focus-number">"[ 03 ]"</span><span class="focus-symbol" aria-hidden="true">"↗"</span>
                    <h3>{move || locale.get().select("公开记录", "Open notes")}<span>" / NOTES"</span></h3>
                    <p>{move || locale.get().select("把实验过程写下来，让下一次探索有迹可循。", "Recording experiments so the next exploration has a trail to follow.")}</p>
                    <span class="focus-foot">"BUILD · LEARN · SHARE"</span>
                </div>
            </div>
        </section>
    }
}
