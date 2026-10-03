<picture>
  <source media="(prefers-color-scheme: dark)" srcset=".github/assets/profile-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset=".github/assets/profile-light.svg">
  <img src=".github/assets/profile-light.svg" alt="MCB / LOG，蓝白与紫黑双色的极简个人主页横幅" width="100%">
</picture>

# MCB / LOG

> 系统编程 · 实时推理 · AI 安全 · 计算机视觉

[ABOUT](#about) / [FOCUS](#focus) / [BLOG DEMO](#blog-demo) / [GITHUB](https://github.com/MCB-SMART-BOY)

---

## About

你好，我是 MCB。喜欢用 Rust 和 Python 探索系统、推理与安全问题，也在 Linux 环境里不断学习、构建和记录。这里展示关注方向与一个正在完善的博客演示；示例文章不代表已经发布的研究或作品。

`Rust` · `Python` · `Linux` · `NixOS` · `Garuda` · `Zed` · `Helix`

## Focus

| 方向 | 关注点 |
| --- | --- |
| 01 / Systems | 系统编程、内核与可靠的软件结构 |
| 02 / Intelligence | 实时推理、计算机视觉与可靠的软件边界 |
| 03 / Notes | 记录实验过程，让下一次探索有迹可循 |

## Blog demo

**MCB / LOG** 是一个以文章为中心的 Rust 全栈博客演示：固定栏目侧栏、独立文章页，以及与 GitHub 明暗主题呼应的蓝白／紫黑双配色。当前仅提供[源码和本地运行方式](https://github.com/MCB-SMART-BOY/MCB-SMART-BOY)，尚未公开部署博客站点。

---

### 项目概览

Rust 编写的博客 demo：Axum 负责 HTTP，Leptos 负责 SSR 页面和浏览器端 WASM hydration；Markdown 正文由 Rust 渲染，页面样式和图标存储于仓库。不需要 Bun/Node，也不从 CDN 加载页面资源。浅色纸感、夜间科幻两套主题；侧栏支持折叠、拖动宽度和移动端抽屉，偏好由浏览器 cookie 保存。

> 这是演示，不是已发布的个人作品集。三篇文章是示例，不代表作者已发表的作品。

### 项目内工具链与启动

需要系统已有的 `rustup` 命令、可用网络及本机编译依赖。`./dev.sh` 将 Rust 1.99.0、WASM 目标、cargo-leptos 0.3.10、wasm-bindgen-cli 0.2.129、Cargo 注册表、编译产物与临时文件安装或生成在仓库的 `run/`；不安装 Bun/Node，不重装或移动系统工具。首次准备时间和空间取决于网络及本机编译环境。

```sh
./dev.sh setup
./dev.sh leptos build
./dev.sh run --locked
```

打开 `http://127.0.0.1:3000`。换端口：`BLOG_ADDR=127.0.0.1:4000 ./dev.sh run --locked`；开发时可运行 `./dev.sh leptos watch`。服务默认仅监听回环地址；不要在未配置 TLS 和部署安全边界前公开暴露。停止服务使用 `Ctrl+C`。

**本项目所有 Cargo 与 Rustup 命令均经 `./dev.sh` 执行**，例如 `./dev.sh test --locked`、`./dev.sh fmt -- --check`、`./dev.sh clippy --all-targets -- -D warnings`、`./dev.sh -- rustc --version`。项目专项验证运行 `./validate.sh`；若安装了 OMP 验证器，再用 `./dev.sh -- "$HOME/.omp/agent/validate.sh"` 执行通用质量门。脚本隔离 `CARGO_HOME`、`RUSTUP_HOME`、`CARGO_TARGET_DIR`、XDG 数据/缓存/配置、`TMPDIR`、Trivy 缓存、Leptos 输出和构建器工具安装目录到 `run/`；直接执行 `cargo`/`rustup` 或外部浏览器不受约束，可能写入用户目录。`run/` 和 `.env` 已忽略，不会提交编译产物和本地设置。其他工具可用 `./dev.sh -- <命令>` 继承隔离变量。

`deny.toml` 显式允许当前依赖使用的宽松许可证；对 Leptos 0.8 间接引入且在当前依赖链中无可直接升级的修补版本的 `paste`、`proc-macro-error2` 两项“停止维护”通告逐项记录例外，其他安全通告仍会导致检查失败。项目维护者须在 2027-01-03 前复查，并在更新 Leptos 后移除不再需要的例外；这不代表依赖没有未来风险。

### 页面与内容

| 路径 | 内容 |
|---|---|
| `/` | 首页、最新文章和入口 |
| `/writing` | 文章归档 |
| `/writing/:slug` | 独立文章；未知 slug 为 404 |
| `/focus` | 三个关注方向 |
| `/about` | 作者与 demo 说明 |

- `src/content.rs` 管理文章顺序、slug、日期及摘要；`content/posts/*.md` 是正文。新增文章时在两处维护数据并重建应用，正文随 Rust 二进制编译；页面更新不依赖在线数据库。
- `src/pages/` 是 Leptos 页面，`src/components/` 是导航、顶栏、侧栏等共用交互；`src/app.rs` 定义路由与 SSR 文档壳，`src/server.rs` 负责 HTTP，`src/preferences.rs` 负责 cookie 和偏好范围；`src/markdown.rs` 将仓库 Markdown 编译为安全 HTML；`assets/site.css` 管理响应式主题，`public/favicon.svg` 是本地图标。
- 首次 HTTP 响应包含可阅读的服务端 HTML；hydration 成功后客户端导航与偏好交互可用。桌面侧栏切页后将键盘焦点移到正文，移动抽屉关闭后焦点返回触发按钮或目标正文。WASM 不可用时移动设备提供纯链接导航；主题切换、侧栏调整需要 WASM。
- 无后台、登录、评论、在线编辑、RSS 或部署配置。Markdown 的原始 HTML 会转义，链接目的地限制为安全协议、站内路径和锚点；作者仍需审阅仓库内容。正式发布前需替换示例文章并评估部署安全、浏览器兼容性和可访问性。

### 设计来源

调研了 [Matklad](https://matklad.github.io/) 的单栏文章、[Anthony Fu](https://antfu.me/posts) 的列表密度、[Dan Luu](https://danluu.com/) 的长文归档、[Tarik Karahodžić](https://www.tarikkarahodzic.dev/projects/personal-site) 的留白，以及 [Stefan Vitasović](https://tympanus.net/codrops/2025/03/05/case-study-stefan-vitasovic-portfolio-2025/) 的几何构图。此 demo 以本地 CSS 构图、大字和清晰的文章层级呈现，不复制第三方素材或加入重型动画；新增浅色编辑排版与暗色科幻主题，尊重系统减少动效偏好。
