# MCB / LOG 项目上下文

本项目是 Rust 全栈个人博客 demo；用户可见的流程：首页 → 文章归档或关注方向 → 文章阅读 / 关于。展示内容为示例，并非作者已发布成果。根目录 `README.md` 前段是 GitHub 用户主页，后段是博客 demo 入门与设计参考；本文件负责代码事实和维护路由。

## 代码与职责

| 修改目标 | 事实源 |
|---|---|
| 文章正文与元数据 | `content/posts/*.md`、`src/content.rs`；顺序和 slug 由代码确定，正文编入二进制 |
| 页面路由与 SSR 文档 | `src/app.rs`、`src/server.rs`、`src/main.rs` |
| 首页、归档、详情、关注领域、关于、404 | `src/pages/` |
| 导航、侧栏、顶栏与移动菜单 | `src/components/` |
| 主题、侧栏宽度与 cookie | `src/preferences.rs` |
| Markdown HTML 安全边界 | `src/markdown.rs` |
| 色彩和布局 / 本地图标 | `assets/site.css` / `public/favicon.svg` |
| Rust 和前端 WASM 构建 / 依赖审计 | `Cargo.toml`、`Cargo.lock`、`dev.sh`、`validate.sh`、`deny.toml` |

架构为 Axum HTTP + Leptos SSR/hydrate（浏览器端 Rust/WASM）；不需要 Node/Bun 或外部页面资产。移动端 SSR 在 WASM 不可用时仍提供真实链接导航；动态主题、折叠、宽度拖动与移动抽屉需要 hydration。首屏主题与侧栏设定由服务端读取 `mcb-ui` cookie；所有来自 cookie 的值须受 `src/preferences.rs` 限制，客户端更新同一字段，避免 SSR/hydration 不一致。Markdown 仅收录仓库内容，未暴露投稿接口；原始 HTML 和不安全链接不被作为可执行页面内容接受。

`Cargo.toml` 的 `disable-erase-components = true` 保持服务端与 WASM 的组件 hydration 标记一致：在真实浏览器中，去掉此设置曾使顶栏按钮失效并触发 `Unrecoverable hydration error`。变更 Leptos 版本或调整组件边界时必须用浏览器重新验证首页、文章与交互；编译和 SSR 路由测试不足以证明 hydration 成功。

`deny.toml` 仅允许已核对的许可证，并针对 Leptos 0.8 间接依赖的 `paste`、`proc-macro-error2` 停止维护通告记录逐项例外（当前依赖链无可直接升级的修补版本）。项目维护者在 2027-01-03 前及升级框架后复核并清理不再需要的例外，不可把它们解释为不存在安全风险。

## 本地流程与验证

1. 首次从项目根运行 `./dev.sh setup`；`run/` 储存 toolchain、Cargo registry、编译与缓存产物，脚本刻意不修改系统工具链。仅经 `./dev.sh` 调用 Cargo/Rustup；直接调用不提供隔离保证。
2. 先运行直接相关的功能和 HTTP 场景、浏览器实际交互，再运行项目验证器 `./validate.sh`（WASM typecheck、SSR/WASM 构建、Rust 测试）。
3. 最后运行通用质量门：`./dev.sh -- "$HOME/.omp/agent/validate.sh"`；其与项目 validator 相互独立。适用 gate 无法通过时明确记录原因，不把跳过当作通过。
4. 除非用户明确授权，不提交、推送、部署或改动个人主页中已核实的资料。用户明确同意的依赖/SSR 迁移取代旧 Askama 模板；不保留旧端点或重复渲染路径。
