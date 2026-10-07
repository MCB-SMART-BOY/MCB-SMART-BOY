use crate::{
    components::profile::{CodeFlowCard, ProjectCards},
    landing::LandingSection,
    locale::Locale,
    profile::{PROFILE_PROJECTS, PROFILE_SUMMARY_EN, PROFILE_SUMMARY_ZH},
};
use leptos::prelude::*;

#[component]
pub(super) fn AboutSection() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <section id=LandingSection::About.id() class="landing-section landing-panel" tabindex="-1" aria-labelledby="about-title">
        <div class="landing-page landing-motion">
        <section class="page-intro about-intro" aria-labelledby="about-title">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" ABOUT / THE PERSON BEHIND THE LOG"</span>
            <div class="profile-intro-grid">
                <div>
                    <h2 id="about-title">{move || locale.get().select("你好，", "Hi, ")}<span class="accent">{move || locale.get().select("我是 MCB.", "I'm MCB.")}</span></h2>
                    <p>{move || locale.get().select(PROFILE_SUMMARY_ZH, PROFILE_SUMMARY_EN)}</p>
                    <a class="inline-link" href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">{move || locale.get().select("访问我的 GitHub ", "Visit my GitHub ")}<span aria-hidden="true">"↗"</span></a>
                </div>
                <img class="profile-logo" src="/images/mcb-logo.png" width="640" height="640" decoding="async"
                    alt=move || locale.get().select("MCB-SMART-BOY 个人 Logo", "MCB-SMART-BOY personal logo")/>
            </div>
            <nav class="profile-jump-links" aria-label=move || locale.get().select("跳转到资料章节", "Jump to profile sections")>
                <a href="/#experience" target="_self">{move || locale.get().select("经历", "Experience")}</a>
                <a href="/#codeflow" target="_self">{move || locale.get().select("社团", "Community")}</a>
            </nav>
        </section>
        </div>
        </section>
    }
}

#[component]
pub(super) fn ProfileSections() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <ProfileProjects locale/>
        <ProfileExperience locale/>
        <ProfileBackground locale/>
        <ProfileCommunity/>
        <ProfileRecognition locale/>
        <p class="profile-note">{move || locale.get().select(
            "文章区目前保留三篇示例内容，不代表已发表的技术文章。",
            "The writing section currently contains three sample pieces, not published technical articles.",
        )}</p>
        <div class="back-link"><a href="/#home" target="_self">{move || locale.get().select("↑ 返回顶部", "↑ Back to top")}</a></div>
    }
}

#[component]
fn ProfileBackground(locale: RwSignal<Locale>) -> impl IntoView {
    view! {
        <section id=LandingSection::Background.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="background-title">
            <div class="landing-motion about-layout">
            <div class="about-label">"BACKGROUND "<span aria-hidden="true">"↘"</span></div>
            <div class="about-text">
                <h3 id="background-title">{move || locale.get().select("从软件工程到系统工具", "From software engineering to systems tools")}</h3>
                <p>{move || locale.get().select(
                    "2024.09 起，本科阶段学习软件工程，关注数据结构、操作系统、数据库与计算机网络。",
                    "Studying software engineering at undergraduate level since September 2024, with a focus on data structures, operating systems, databases, and computer networks.",
                )}</p>
                <dl class="profile-skills">
                    <div>
                        <dt>{move || locale.get().select("语言与后端", "Languages & backend")}</dt>
                        <dd>{move || locale.get().select(
                            "Rust（Tokio、Actix-Web、SQLx、egui）、Python（FastAPI、PyTorch）、OCaml",
                            "Rust (Tokio, Actix-Web, SQLx, egui), Python (FastAPI, PyTorch), OCaml",
                        )}</dd>
                    </div>
                    <div>
                        <dt>{move || locale.get().select("系统与环境", "Systems & environment")}</dt>
                        <dd>{move || locale.get().select(
                            "Linux / NixOS、Nix、Helix / Zed、可复现开发环境",
                            "Linux / NixOS, Nix, Helix / Zed, reproducible development environments",
                        )}</dd>
                    </div>
                    <div>
                        <dt>{move || locale.get().select("验证与 AI", "Verification & AI")}</dt>
                        <dd>{move || locale.get().select(
                            "eBPF、控制流分析、Lean 4 / Isabelle、实时推理与计算机视觉",
                            "eBPF, control-flow analysis, Lean 4 / Isabelle, real-time inference and computer vision",
                        )}</dd>
                    </div>
                </dl>
                <p>{move || locale.get().select(
                    "围绕开源工具链与跨平台分发持续实践。",
                    "Continuing to work on open-source tooling and cross-platform distribution.",
                )}</p>
            </div>
            </div>
        </section>
    }
}

#[component]
fn ProfileProjects(locale: RwSignal<Locale>) -> impl IntoView {
    view! {
        <section id=LandingSection::Projects.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="projects-title">
            <div class="landing-motion">
            <h3 id="projects-title">{move || locale.get().select("项目与开源", "Projects & open source")}</h3>
            <p>{move || locale.get().select("n3v3（原 Neve）", "n3v3 (formerly Neve)")}</p>
            <ProjectCards projects=&PROFILE_PROJECTS/>
            </div>
        </section>
    }
}

#[component]
fn ProfileExperience(locale: RwSignal<Locale>) -> impl IntoView {
    view! {
        <section id=LandingSection::Experience.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="experience-title">
            <div class="landing-motion">
            <h3 id="experience-title">{move || locale.get().select("工程与研究经历", "Engineering & research experience")}</h3>
            <ul>
                <li>
                    <h4>{move || locale.get().select("形式化验证实习", "Formal verification internship")}</h4>
                    <p>{move || locale.get().select("2026.06 — 至今", "June 2026 — present")}</p>
                    <p>{move || locale.get().select(
                        "围绕 Lean 4 与大语言模型，构建证明尝试、类型检查和反馈迭代流程。",
                        "Working with Lean 4 and language models on proof attempts, type checking, and feedback-driven iteration.",
                    )}</p>
                </li>
                <li>
                    <h4>{move || locale.get().select("工业异常检测合作项目", "Industrial anomaly detection collaboration")}</h4>
                    <p>{move || locale.get().select("2026.03 — 2026.06", "March — June 2026")}</p>
                    <p>{move || locale.get().select(
                        "使用 FastAPI、Vue 3 / TypeScript 与 PyTorch，参与异常检测平台、实时事件推送及模型适配。",
                        "Contributed to an anomaly detection platform, real-time event delivery, and model integration using FastAPI, Vue 3 / TypeScript, and PyTorch.",
                    )}</p>
                </li>
                <li>
                    <h4>{move || locale.get().select("计算机视觉与实时告警后端", "Computer vision & real-time alerting")}</h4>
                    <p>{move || locale.get().select(
                        "构建模型服务与异步后端，探索 Python 到 Rust 的模块迁移，以及 WebSocket 与多数据库集成。",
                        "Built model services and asynchronous backends, exploring migration from Python to Rust, WebSocket delivery, and multi-database integration.",
                    )}</p>
                </li>
            </ul>
            </div>
        </section>
    }
}

#[component]
fn ProfileCommunity() -> impl IntoView {
    view! {
        <section id=LandingSection::Community.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="codeflow-title">
            <div class="landing-motion">
            <h3 id="codeflow-title">"CodeFlow·代码灵动"</h3>
            <CodeFlowCard/>
            </div>
        </section>
    }
}

#[component]
fn ProfileRecognition(locale: RwSignal<Locale>) -> impl IntoView {
    view! {
        <section id=LandingSection::Recognition.id() class="landing-section profile-section" tabindex="-1" aria-labelledby="recognition-title">
            <div class="landing-motion">
            <h3 id="recognition-title">{move || locale.get().select("荣誉与交流", "Awards & exchanges")}</h3>
            <h4>{move || locale.get().select("荣誉", "Awards")}</h4>
            <ul>
                <li>{move || locale.get().select("2024 创青春创业大赛国家级三等奖", "2024 national third prize in the Chuang Qing Chun entrepreneurship competition")}</li>
                <li>{move || locale.get().select("2024 蓝桥杯 C/C++ 组省级二等奖", "2024 provincial second prize in the Lan Qiao Cup C/C++ category")}</li>
                <li>{move || locale.get().select("2025 校级优秀学生奖学金", "2025 university-level outstanding student scholarship")}</li>
                <li>{move || locale.get().select("2026 大学生计算机设计大赛项目负责人、国家级二等奖", "2026 project lead, national second prize in the Chinese Collegiate Computing Competition")}</li>
            </ul>
            <h4>{move || locale.get().select("交流", "Exchange")}</h4>
            <ul>
                <li>{move || locale.get().select("2025 中国软件大会形式化验证专题交流", "2025 formal verification discussions at the China Software Conference")}</li>
            </ul>
            </div>
        </section>
    }
}
