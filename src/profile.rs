pub(crate) struct ProfileProject {
    pub(crate) name: &'static str,
    pub(crate) url: &'static str,
    pub(crate) summary_zh: &'static str,
    pub(crate) summary_en: &'static str,
    pub(crate) stack: &'static str,
}

pub(crate) const GRIDIX_URL: &str = "https://github.com/MCB-SMART-BOY/Gridix";
pub(crate) const N3V3_URL: &str = "https://github.com/MCB-SMART-BOY/n3v3";
pub(crate) const VERIFIER_RS_URL: &str = "https://github.com/MCB-SMART-BOY/verifier-rs";
pub(crate) const ISABELLE_RS_URL: &str = "https://github.com/MCB-SMART-BOY/isabelle-rs";
pub(crate) const MINIF2F_URL: &str = "https://github.com/MCB-SMART-BOY/minif2f";

pub(crate) const PROFILE_SUMMARY_ZH: &str = "软件工程本科在读，使用 Rust 与 Python，探索系统软件、语言工具链和形式化验证。CodeFlow·代码灵动创办者。";
pub(crate) const PROFILE_SUMMARY_EN: &str = "Software engineering undergraduate building with Rust and Python, exploring systems software, language tooling, and formal verification. Founder of CodeFlow.";

pub(crate) const PROFILE_PROJECTS: [ProfileProject; 5] = [
    ProfileProject {
        name: "Gridix",
        url: GRIDIX_URL,
        summary_zh: "键盘优先的跨平台数据库管理工具，连接 SQLite、PostgreSQL 与 MySQL/MariaDB。",
        summary_en: "A keyboard-first, cross-platform database tool for SQLite, PostgreSQL, and MySQL/MariaDB.",
        stack: "Rust · egui · SQLx",
    },
    ProfileProject {
        name: "n3v3",
        url: N3V3_URL,
        summary_zh: "面向系统自动化的带类型语言，围绕配置、构建与脚本提供语言及工具链。",
        summary_en: "A typed language and toolchain for system automation, configuration, builds, and scripting.",
        stack: "Rust · Language tooling",
    },
    ProfileProject {
        name: "verifier-rs",
        url: VERIFIER_RS_URL,
        summary_zh: "以 Rust 探索 BPF 程序验证，拆分验证核心与平台适配。",
        summary_en: "Exploring BPF program verification in Rust, separating the verification core from platform integration.",
        stack: "Rust · BPF · Systems",
    },
    ProfileProject {
        name: "isabelle-rs",
        url: ISABELLE_RS_URL,
        summary_zh: "探索 Isabelle 风格的证明内核与 Isar 证明流程，关注证明结果及其信任边界。",
        summary_en: "Exploring an Isabelle-style proof kernel and Isar workflows, with explicit attention to proof results and trust boundaries.",
        stack: "Rust · Isabelle · LCF",
    },
    ProfileProject {
        name: "miniF2F",
        url: MINIF2F_URL,
        summary_zh: "用 Rust 与 vLLM 编排 Lean 4 模型的证明尝试与校验流程。",
        summary_en: "Orchestrating Lean 4 proof attempts and checking workflows with Rust and vLLM.",
        stack: "Rust · Lean 4 · vLLM",
    },
];
