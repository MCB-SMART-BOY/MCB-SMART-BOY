use crate::{
    locale::Locale,
    profile::{GRIDIX_URL, ISABELLE_RS_URL, MINIF2F_URL, N3V3_URL, VERIFIER_RS_URL},
};
use leptos::prelude::*;

#[component]
pub(super) fn FocusSection() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <section class="page-intro focus-intro" aria-labelledby="focus-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" 03 / CURRENT SIGNAL"</span>
            <h2 id="focus-title">{move || locale.get().select("持续关注", "Areas of focus")}<span class="accent">"."</span></h2>
            <p>{move || locale.get().select("不是完成清单，而是持续探索的问题空间。", "Not a list of achievements, but questions to keep exploring.")}</p>
        </section>
        <section class="section focus-section" aria-labelledby="focus-cards-title">
            <h2 class="sr-only" id="focus-cards-title">{move || locale.get().select("关注方向", "Areas of focus")}</h2>
            <div class="section-top"><span class="section-kicker">"WHERE CURIOSITY GOES"</span></div>
            <div class="focus-grid">
                <div class="focus-card">
                    <span class="focus-number">"[ 01 ]"</span><span class="focus-symbol" aria-hidden="true">"⌘"</span>
                    <h3>{move || locale.get().select("系统与开发工具", "Systems & developer tools")}<span>" / SYSTEMS & TOOLS"</span></h3>
                    <p>{move || locale.get().select("用 Rust 探索数据库工具、语言实现与 eBPF 验证。", "Exploring database tools, language implementation, and eBPF verification with Rust.")}</p>
                    <span class="focus-foot">"RUST · DATABASES · EBPF"</span>
                    <div class="focus-project-links">
                        <a href=GRIDIX_URL target="_blank" rel="noopener noreferrer">"Gridix ↗"</a>
                        <a href=N3V3_URL target="_blank" rel="noopener noreferrer">"n3v3 ↗"</a>
                        <a href=VERIFIER_RS_URL target="_blank" rel="noopener noreferrer">"verifier-rs ↗"</a>
                    </div>
                </div>
                <div class="focus-card">
                    <span class="focus-number">"[ 02 ]"</span><span class="focus-symbol" aria-hidden="true">"✳"</span>
                    <h3>{move || locale.get().select("形式化验证", "Formal verification")}<span>" / FORMAL VERIFICATION"</span></h3>
                    <p>{move || locale.get().select("围绕 Lean 4、Isabelle 与证明内核，探索推理流程和信任边界。", "Exploring reasoning workflows and trust boundaries with Lean 4, Isabelle, and proof kernels.")}</p>
                    <span class="focus-foot">"LEAN 4 · ISABELLE · LCF"</span>
                    <div class="focus-project-links">
                        <a href=ISABELLE_RS_URL target="_blank" rel="noopener noreferrer">"isabelle-rs ↗"</a>
                        <a href=MINIF2F_URL target="_blank" rel="noopener noreferrer">"miniF2F ↗"</a>
                    </div>
                </div>
                <div class="focus-card">
                    <span class="focus-number">"[ 03 ]"</span><span class="focus-symbol" aria-hidden="true">"↗"</span>
                    <h3>{move || locale.get().select("AI 与工程实践", "Applied AI & engineering")}<span>" / APPLIED AI"</span></h3>
                    <p>{move || locale.get().select("关注视觉推理、异常检测、异步服务与可复现部署。", "Working on visual inference, anomaly detection, asynchronous services, and reproducible deployment.")}</p>
                    <span class="focus-foot">"PYTORCH · FASTAPI · NIX"</span>
                    <div class="focus-project-links">
                        <a href="/#experience" target="_self">{move || locale.get().select("查看工程经历", "View engineering experience")}</a>
                    </div>
                </div>
            </div>
        </section>
    }
}
