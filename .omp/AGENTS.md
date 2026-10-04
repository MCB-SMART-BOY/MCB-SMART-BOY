# MCB / LOG 项目上下文

本项目是 Rust 全栈个人博客 demo；用户可见的流程：首页 → 文章归档或关注方向 → 文章阅读 / 关于。三篇文章为示例，并非作者已发布成果。根目录 `README.md` 仅用于 GitHub 用户自我介绍，不承载博客介绍、示例文章声明或折叠的项目手册。个人资料以用户已有介绍为准，不虚构履历与成果；博客浅色主题参考 `resume-template` 的暖纸色、细网格与靛蓝点缀，深色保留紫黑科幻主题。本文件负责博客运行说明、代码事实、设计参考和维护路由。

## 代码与职责

| 修改目标 | 事实源 |
|---|---|
| 文章正文与元数据 | `content/posts/*.md`、`src/content.rs`；顺序、slug、阅读分钟数由文章表确定，书籍、章节及文章归属由 `BOOKS` 确定，正文编入二进制 |
| 页面路由与 SSR 文档 | `src/app.rs`、`src/reading.rs`、`src/server.rs`、`src/main.rs` |
| 首页、归档、详情、关注领域、关于、404 | `src/pages/` |
| 导航、侧栏、顶栏、面包屑、分享与移动菜单 | `src/components/` |
| 主题、侧栏宽度与 cookie | `src/preferences.rs`；语言及 `mcb-lang` cookie 见 `src/locale.rs` |
| Markdown HTML 安全边界 | `src/markdown.rs` |
| 色彩和布局 / 本地字体与许可 / 本地图标 | `assets/site.css` / `public/fonts/` / `public/favicon.svg` |
| Rust 和前端 WASM 构建 / 依赖审计 | `Cargo.toml`、`Cargo.lock`、`dev.sh`、`validate.sh`、`deny.toml` |
| GitHub 用户自我介绍 / 本地图形资源 | `README.md` / `.github/assets/profile-*.svg`；双色横幅、终端卡片、兴趣卡片和技术图标 |

架构为 Axum HTTP + Leptos SSR/hydrate（浏览器端 Rust/WASM）；不需要 Node/Bun 或外部页面资产。移动端 SSR 在 WASM 不可用时仍提供真实链接导航；动态主题、折叠、宽度拖动、移动抽屉、分享与语言选择需要 hydration。首屏主题与侧栏设定由服务端读取 `mcb-ui` cookie，语言由 `mcb-lang` cookie 读取，合法值仅 `zh-CN` 和 `en`，默认中文。服务端渲染 `<html lang>` 及同源 bootstrap 数据，客户端使用同一初值避免 SSR/hydration 不一致。语言切换只翻译固定界面文案，三篇示例文章正文与标题依旧是中文原文（`lang="zh-CN"` 并标注原文）；不要把英文 UI 误报为英文版文章。所有来自 cookie 的值须受解析器限制，客户端仅更新对应字段。Markdown 仅收录仓库内容，未暴露投稿接口；原始 HTML 和不安全链接不被作为可执行页面内容接受。

`Cargo.toml` 的 `disable-erase-components = true` 保持服务端与 WASM 的组件 hydration 标记一致：在真实浏览器中，去掉此设置曾使顶栏按钮失效并触发 `Unrecoverable hydration error`。变更 Leptos 版本或调整组件边界时必须用浏览器重新验证首页、文章与交互；编译和 SSR 路由测试不足以证明 hydration 成功。

顶栏左侧为可回溯的当前位置面包屑，中间为 `MCB-SMART-BOY`，右侧顺序由左到右为语言、主题、GitHub、分享；文章路径只依据 `src/content.rs` 的书籍／章节关系生成，不从 slug 猜测栏目，段落目录仅从 Markdown 标题生成。桌面折叠键展开时位于 Logo 右侧；点击后真实按钮立即退出交互，装饰图标按测量坐标缩小飞入 Logo，抵达后才出现圆形同心水波，收起后仅保留 Logo 展开入口。首次以折叠偏好加载、悬停或聚焦不触发水波，快速展开会取消装饰，减少动态效果时立即切换。移动端使用顶栏菜单键。分享优先调用浏览器 Web Share，不支持时复制当前完整 URL，权限/能力失败时展示可手动复制的真实链接；不把失败伪装为成功。

`deny.toml` 仅允许已核对的许可证，并针对 Leptos 0.8 间接依赖的 `paste`、`proc-macro-error2` 停止维护通告记录逐项例外（当前依赖链无可直接升级的修补版本）。项目维护者在 2027-01-03 前及升级框架后复核并清理不再需要的例外，不可把它们解释为不存在安全风险。

## GitHub 个人页视觉维护

`README.md` 使用 GitHub 支持的 HTML、`picture` 和本地 SVG 组织整页：个人横幅 → 简短中文介绍 → 技术图标 → 开发环境与关注方向卡片 → 仓库入口。不恢复博客手册，不添加未经核实的职业头衔、贡献统计或项目成果。

- `profile-light.svg`、`profile-dark.svg` 是原创横幅，沿用博客配色、网格与几何轨道。CSS 动效仅用于装饰性的轨道点和光标；`prefers-reduced-motion: reduce` 时关闭，不依赖动画才能读到内容。SVG 动效的播放取决于 GitHub 和浏览器，不能承诺所有客户端一致。
- `profile-terminal-*.svg` 以最初 README 的 `fastfetch` 为视觉参考，记录 NixOS / Garuda Linux、Rust / Python、Zed / Helix；`profile-focus-*.svg` 展示系统与内核、实时推理与视觉、AI 安全。个人资料修改时同时更新卡片文字、`title`/`desc` 与 README 的替代文本。
- 两张卡片使用固定像素宽度和透明画布留白，不用 Markdown 表格强制双列；窄屏依靠 GitHub 图片宽度限制自然换行。验收必须检查完整 GitHub 样式中的桌面、390px 和 320px 布局，两种主题、资源加载与减少动态效果，不能只检查图片文件或无样式 Markdown。
- `profile-tools-*.svg` 是 2026-10-03 从 [Skill Icons](https://github.com/tandpfun/skill-icons) 的 `rust,python,linux,nix` 两色输出保存的本地图标，`metadata` 内保留来源和完整 MIT 许可。渲染不请求第三方图标或统计服务；更新资源时须重新核对来源、许可和 SVG 的自包含性，不接受脚本、事件处理器、外部引用或 `foreignObject`。
- 社区参考为 [Orhun](https://github.com/orhun/orhun) 的终端主题、[DenverCoder1](https://github.com/DenverCoder1/DenverCoder1) 的图标分组、[Anurag Hazra](https://github.com/anuraghazra/anuraghazra) 的视觉层次与 [Sindre Sorhus](https://github.com/sindresorhus/sindresorhus) 的个人符号表达；另参考本仓库最初的个人 README。仅借鉴构图方法，不复制他人的履历、动图或统计数据。

## 项目内工具链与启动

需要系统已有的 `rustup` 命令、可用网络及本机编译依赖。`./dev.sh` 将 Rust 1.99.0、WASM 目标、cargo-leptos 0.3.10、wasm-bindgen-cli 0.2.129、Cargo 注册表、编译产物与临时文件安装或生成在仓库的 `run/`；不安装 Bun/Node，不重装或移动系统工具。首次准备时间和空间取决于网络及本机编译环境。

`Cargo.toml` 的开发配置使用 `debug = 1` 保留有限调试信息和行号；完整调试信息曾使 Leptos 单态化产物超过链接器 4 GiB 调试节限制并报 `R_X86_64_32 out of range`。这不改变发布配置，但开发调试不再包含完整变量/类型信息；不要用关闭 hydration 一致性设置来规避链接问题。

```sh
./dev.sh setup
./dev.sh leptos build
./dev.sh run --locked
```

打开 `http://127.0.0.1:3000`。换端口：`BLOG_ADDR=127.0.0.1:4000 ./dev.sh run --locked`；开发时可运行 `./dev.sh leptos watch`。服务默认仅监听回环地址；不要在未配置 TLS 和部署安全边界前公开暴露。停止服务使用 `Ctrl+C`。

**本项目所有 Cargo 与 Rustup 命令均经 `./dev.sh` 执行**，例如 `./dev.sh test --locked`、`./dev.sh fmt -- --check`、`./dev.sh clippy --all-targets -- -D warnings`、`./dev.sh -- rustc --version`。脚本隔离 `CARGO_HOME`、`RUSTUP_HOME`、`CARGO_TARGET_DIR`、XDG 数据/缓存/配置、`TMPDIR`、Trivy 缓存、Leptos 输出和构建器工具安装目录到 `run/`；直接执行 `cargo`/`rustup` 或外部浏览器不受约束，可能写入用户目录。`run/` 和 `.env` 已忽略，不会提交编译产物和本地设置。其他工具可用 `./dev.sh -- <命令>` 继承隔离变量。

## 验证与维护

1. 先运行直接相关的功能和 HTTP 场景、浏览器实际交互，再运行项目验证器 `./validate.sh`（WASM typecheck、SSR/WASM 构建、Rust 测试）。
2. 最后运行通用质量门：`./dev.sh -- "$HOME/.omp/agent/validate.sh"`；其与项目 validator 相互独立。适用 gate 无法通过时明确记录原因，不把跳过当作通过。
3. 除非用户明确授权，不提交、推送、部署或改动个人主页中已核实的资料。用户明确同意的依赖/SSR 迁移取代旧 Askama 模板；不保留旧端点或重复渲染路径。

## 页面与内容

| 路径 | 内容 |
|---|---|
| `/` | 首页、最新文章和入口 |
| `/writing` | 书籍合集与实际存在的独立文章 |
| `/writing/books/:book_slug` | 书籍简介与章节入口；未知书籍为 404 |
| `/writing/books/:book_slug/chapters/:chapter_slug` | 章节标题、文章数与本章文章入口；未知章节为 404 |
| `/writing/:slug` | 文章详情与段落锚点；未知 slug 为 404 |
| `/focus` | 三个关注方向 |
| `/about` | 作者与 demo 说明 |

新增文章时在 `src/content.rs` 维护文章顺序、slug、日期、阅读分钟数和摘要，在 `BOOKS` 维护书籍、章节及文章 slug 归属，在 `content/posts/` 增加正文并重建应用；正文随 Rust 二进制编译，页面更新不依赖在线数据库。当前内容目录不是文件系统自动扫描结果；侧栏、正文索引、返回链接和面包屑共享这套内容关系，不建立第二份导航目录。文章 Markdown 标题同时生成安全 HTML 锚点与目录，首次 HTTP 响应包含可阅读的服务端 HTML；hydration 成功后客户端导航与偏好交互可用。

侧栏保留独立的主导航区和文章区，文章区依次分为书籍列表、单本书的章节、单章的文章、单篇文章的段落，构成 `Main → Books → Book → Chapter → Article → 段落` 阅读路径。点击、无修饰滚轮或方向键／页键进入文章区时，侧栏和正文同步切到 `/writing`；主导航与书籍列表之间保留一次快速纵向切换，滚轮返回主导航时打开首页，惯性尾部被消费。折叠栏的文章入口先展开侧栏，再进入书籍总览，不重复写入历史。

选择书籍后横向左划进入该书章节，同时打开 `/writing/books/:book_slug`；选择章节进入章节页及本章文章列表，选择文章进入正文及真实 Markdown 标题生成的段落链接。各级顶部集中放置返回上级、当前位置和下级列表说明；短视口中头部固定，只有当前面板内部溢出内容滚动。书籍页展示本书引言与章节入口，章节页只展示章节信息和本章文章，不重复书籍引言。桌面已 hydration 且侧栏展开时，段落目录仅保留在侧栏；移动端、折叠侧栏或无 JavaScript 时，正文保留原生折叠目录。

只有书籍列表列出书籍，章节面板只列章节，文章面板只列该章节所属文章；独立文章保留单独入口。深链按真实书籍／章节／文章关系恢复对应面板，不推断不存在的归属。Alt/Ctrl/Meta/Shift 修饰的键盘与滚轮操作保持浏览器原义；隐藏面板设为 `inert`，显式翻页及历史导航时若焦点原在侧栏则移到目标标题，滚轮不抢正文焦点。书籍／章节层级链接保持移动抽屉打开以呈现切页；选择文章或段落（包括当前文章）后关闭抽屉并聚焦正文，其余关闭操作返回菜单按钮。桌面普通链接导航将焦点移到正文，折叠和语言切换不重置面板。

博客采用浅色暖纸细网格与夜间科幻两套主题，采用小圆角控件、圆角卡片；侧栏支持折叠、拖动宽度和移动端抽屉。字体由 `assets/site.css` 的 `@font-face` 自托管 Maple Mono NF、JetBrainsMono Nerd Font Mono、思源黑体与思源宋体 Regular/Bold；微软雅黑与仿宋仅为访客系统字体回退，不重新分发。`public/fonts/NOTICE.txt` 记录来源、SHA 与许可证路径；完整 CJK OTF 较大，首屏需要按需加载字体，在调整字体打包方案时核对字形覆盖与许可。无后台、登录、评论、在线编辑、RSS 或部署配置。正式发布前需替换示例文章、审阅仓库内容，并评估部署安全、浏览器兼容性和可访问性。

## 博客设计来源

调研了 [Matklad](https://matklad.github.io/) 的单栏文章、[Anthony Fu](https://antfu.me/posts) 的列表密度、[Dan Luu](https://danluu.com/) 的长文归档、[Tarik Karahodžić](https://www.tarikkarahodzic.dev/projects/personal-site) 的留白，以及 [Stefan Vitasović](https://tympanus.net/codrops/2025/03/05/case-study-stefan-vitasovic-portfolio-2025/) 的几何构图。浅色与文章背景借鉴本账号 [resume-template](https://github.com/MCB-SMART-BOY/resume-template) 中 `assets/styles/resume.css` 的暖纸底色、20px 浅网格、靛蓝点缀和正文深灰，而非复制用于 A4 导出的整张背景图。博客以本地 CSS 构图、大字和清晰的文章层级呈现，不复制第三方素材或加入重型动画，尊重系统减少动效偏好。
