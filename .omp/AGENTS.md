# MCB / LOG 项目上下文

本项目是 Rust 静态个人博客：构建时生成完整 HTML，浏览器使用 Rust/WASM 接管交互；Axum 保留为构建和本地开发工具，不是线上服务。首页、关于、关注领域按顺序组成同一份可自然滚动的文档，真实项目、社团与公开经历作为独立资料章节展示，文章归档和阅读保持独立路由。三篇文章为示例，并非作者已发布成果，始终与真实项目分开展示。根目录 `README.md` 仅用于简短的 GitHub 视觉和技术栈展示，不承载教育、经历、社团、荣誉、详细项目资料或博客手册；不得将博客个人资料自动同步到 README。个人资料以用户提供并批准公开的内容为准，不虚构履历与成果；博客两种主题共用暖纸细网格的布局与装饰，浅色参考 `resume-template`，深色保留紫黑配色，不另设一套版式。本文件负责博客运行说明、代码事实、设计参考和维护路由。

个人资料仅公开 `MCB` / `MCB-SMART-BOY` 与 GitHub；经历保留脱敏技术内容，不记录或发布真实姓名、联系方式、证件照、完整简历、合作单位及内部业务数据。教育不补造学校，项目不宣称未经核实的性能或证明成功率。CodeFlow·代码灵动为作者创办的社团，B站入口使用用户明确提供的 [社团主页](https://space.bilibili.com/3493283751791185)，不增加视频嵌入、二维码或账号查询。外站访问曾返回验证码，不能将链接配置正确写成账号内容核验通过。荣誉与会议交流分别表述。

## 代码与职责

| 修改目标 | 事实源 |
|---|---|
| 内容目录、文章正文与元数据 | `content/posts/**` 的真实位置、`_index.md` 和文章 front matter；作者规范见 [content-format.md](content-format.md) |
| 构建内容索引与通用查询 | `build.rs`、`build_support/`、`src/content.rs`；不手写文章表或归属表 |
| 页面路由与 SSR 文档 | `src/app.rs`、`src/reading.rs`、`src/server.rs`、`src/main.rs` |
| 静态导出、资源校验与旧产物替换 | `src/export.rs`、`src/export/assets.rs`、`src/export/output.rs`；页面清单来自 `src/content.rs::page_paths` |
| 单页首页、关于、关注与资料章节 / 归档、详情、404 | `src/pages/`；根文档由 `home.rs` 按正文顺序组合；`about.rs` 分别提供关于简介 `AboutSection` 和资料章节 `ProfileSections`，不单独注册关注和关于路由 |
| 单页可见分区、导航状态与滚动过渡 | `src/landing.rs`、`src/landing_motion.rs`；由 `SiteShell` 提供共享活动状态，侧栏和面包屑共同使用；动效样式见 `assets/site.css` |
| 公开简介与项目事实 | `src/profile.rs`；静态中英资料，不进入文章索引，不在运行时拉取 GitHub |
| 博客项目与社团展示 / 品牌图片 | `src/components/profile.rs` / `public/images/mcb-logo.png`、`codeflow-logo.png`；640×640 去元数据 PNG，保留完整图形与白底；个人 Logo 也用于侧栏展开、折叠与移动菜单，不替换 favicon 或 README 资源 |
| 导航、侧栏、顶栏、面包屑、分享与移动菜单 | `src/components/` |
| 阅读树 / 文章目录与滚动高亮 / 响应式焦点 | `src/components/reading_navigation.rs`、`reading_directory.rs` / `current_directory.rs`、`article_outline.rs` / `directory_focus.rs` |
| 主题、侧栏宽度与 cookie / 静态首屏恢复 | `src/preferences.rs`、`src/components/shell.rs`、`src/lib.rs`；语言及 `mcb-lang` cookie 见 `src/locale.rs` |
| Markdown HTML 安全边界 | `src/markdown.rs` |
| 色彩和布局 / 本地字体与许可 / 本地图标 | `assets/site.css` / `public/fonts/` / `public/favicon.svg` |
| Rust 和前端 WASM 构建 / 静态发布 / 依赖审计 | `Cargo.toml`、`Cargo.lock`、`dev.sh`、`export.sh`、`validate.sh`、`deny.toml`；独立 Pages 仓库的安装配置见 `.omp/pages/workflow.yml` |
| 路由依赖的历史导航补丁 | `vendor/leptos_router/src/location/history.rs`；由根 `Cargo.toml` 的 `[patch.crates-io]` 选择 |
| GitHub 用户自我介绍 / 本地图形资源 | `README.md` / `.github/assets/profile-*.svg`；双色横幅、终端卡片、兴趣卡片和技术图标 |

发布流程为构建时 Axum/Leptos SSR → 完整静态 HTML、CSS、JS/WASM 和本地资源 → 浏览器 hydration。线上不运行 Axum，也不引入 Node/Bun 服务或外部页面资产。本地仍可使用 SSR 服务开发。无 JavaScript 或 WASM 加载失败时，正文、真实页面链接和原生文章目录仍可用；动态主题、折叠、宽度拖动、移动抽屉、分享与语言选择需要 hydration。

静态 HTML 统一使用中文、浅色、展开侧栏和 248px 宽度；`data-ssr-ui` / `data-ssr-lang` 保存不可变初值，客户端必须用它们完成首次 hydration。文档头部脚本在 CSS 前校验 `mcb-ui`，只提前恢复色彩，不翻译文案、不改变组件结构。接管后一次性读取并校验 `mcb-ui` / `mcb-lang`，恢复主题、语言、折叠和宽度；不能用默认值覆盖保存的 cookie，也不能触发折叠飞入或水波。语言合法值仅 `zh-CN` 和 `en`；无效值或读取失败使用默认值。英文偏好可能短暂显示中文 UI，这是单份静态 HTML 的取舍，不是按 cookie 个性化的服务端首屏。本地 SSR 开发仍支持请求 cookie 初值。

语言切换只翻译固定界面文案，三篇示例文章正文与标题依旧是中文原文（`lang="zh-CN"` 并标注原文）；不要把英文 UI 误报为英文版文章。所有来自 cookie 的值须受解析器限制，客户端仅更新对应字段。Markdown 仅收录仓库内容，未暴露投稿接口；原始 HTML 和不安全链接不被作为可执行页面内容接受。

`Cargo.toml` 的 `disable-erase-components = true` 保持服务端与 WASM 的组件 hydration 标记一致：在真实浏览器中，去掉此设置曾使顶栏按钮失效并触发 `Unrecoverable hydration error`。变更 Leptos 版本或调整组件边界时必须用浏览器重新验证首页、文章与交互；编译和 SSR 路由测试不足以证明 hydration 成功。

`src/lib.rs` 的 `recursion_limit = "256"` 只提高编译期视图布局查询深度：合并三个完整分区后，默认深度已实际报 `queries overflow the depth limit`。不通过运行时装箱或关闭 hydration 一致性设置绕过它。

顶栏不再显示居中的 `MCB-SMART-BOY`，当前位置面包屑占据操作组之外的剩余宽度；窄桌面与移动端面包屑独占下一行。宽桌面、窄桌面、移动端的实际页头高度分别为 72px、100px、96px，须与 `--topbar-height` 和锚点偏移同步。右侧顺序由左到右为语言、主题、GitHub、CodeFlow B站、分享；外部社交链接在新窗口打开并设置 `noopener noreferrer`。内容路径和祖先只依据文件扫描生成的目录索引，不从标签猜测栏目，段落目录仅从 Markdown 标题生成。侧栏顶部品牌容器高 68px，Logo 与折叠／展开按钮均为 44×44px；移动抽屉沿用相同 Logo 尺寸。桌面折叠键展开时位于完整个人 Logo 右侧；点击后真实按钮立即退出交互，先捕获按钮起点，再由可取消帧测量折叠后的 Logo 中心并启动飞入，抵达后才出现圆形同心水波，不复用展开态 Logo 的旧中心。收起后仅保留 Logo 展开入口。首次以折叠偏好加载、悬停或聚焦不触发水波，快速展开会取消装饰和待处理帧，减少动态效果时立即切换。移动端使用顶栏菜单键。分享优先调用浏览器 Web Share，不支持时复制当前完整 URL，权限/能力失败时展示可手动复制的真实链接；不把失败伪装为成功。

`deny.toml` 仅允许已核对的许可证，并针对 Leptos 0.8 间接依赖的 `paste`、`proc-macro-error2` 停止维护通告记录逐项例外（当前依赖链无可直接升级的修补版本）。项目维护者在 2027-01-03 前及升级框架后复核并清理不再需要的例外，不可把它们解释为不存在安全风险。

### 路由依赖补丁

`vendor/leptos_router` 固定为 `0.8.16`，来自官方发行包及上游提交 [`584c3a2d884b0e4dba9e3f822a9b6982cff072c7`](https://github.com/leptos-rs/leptos/tree/584c3a2d884b0e4dba9e3f822a9b6982cff072c7/router)。保留原 manifest、源码、README、构建脚本和 `.cargo_vcs_info.json`；MIT `LICENSE` 取自该提交根目录。除 `src/location/history.rs` 外，复制的发行包文件保持原样；局部 formatter 配置只采用上游的稳定选项。原始 `history.rs` 的 SHA-256 为 `d668d8477c2fa4324423b0df9c95dbc13fe08ea5bf2d31e4bde3b7ebd9c25e42`。

已复现的上游缺陷是 Back→Forward 后内部 `path_stack` 未同步，再点原目录时正文改变、地址栏不变。补丁以浏览器实际完整 URL（含 query/hash）决定是否 push；popstate 同步当前栈顶，replace 不追加历史项，浏览器提交错误保留原始 `JsValue` 并记录，不新增 panic。内部栈仍只是路由过渡动画的方向估计，不能区分重复 URL 的真实历史游标；本站未启用该框架 transition，地址提交不依赖此估计。

维护者升级框架时须先检查上游是否真正修复，再移除补丁并复跑浏览器回归：`/writing/`→系统目录→Rust 目录→Rust 文章→Back→Forward→Rust 目录；检查网址、正文、面包屑、左树和随后 Back 的目标一致。同时覆盖两页往返、重复 URL、多步历史、query/hash、相同完整 URL 去重及 replace。现有 Rust/SSR 测试不执行浏览器 History API，不能代替这组实际交互。禁止修改 `run/cargo/registry` 缓存充当修复，或在应用侧补写网址掩盖分叉。

本地 path 源码不受 `Cargo.lock` 的 registry checksum 校验；`--locked` 只锁定依赖图。补丁源码必须保留在版本审查、密钥扫描和依赖审计范围内，不因放入 `vendor/` 而排除。

## GitHub 个人页视觉维护

`README.md` 使用 GitHub 支持的 HTML、`picture` 和本地 SVG 组织整页：个人横幅 → 简短中文介绍 → 技术图标 → 开发环境与关注方向卡片 → 仓库入口。仅维护美观、技术栈和简短兴趣表达；用户明确不在此公开详细个人资料。博客的项目清单、教育、经历、社团、荣誉和新增个人/社团 Logo 不接入 README。不恢复博客手册，不添加职业头衔或贡献统计。

- `profile-light.svg`、`profile-dark.svg` 是原创横幅，沿用博客配色、网格与几何轨道。CSS 动效仅用于装饰性的轨道点和光标；`prefers-reduced-motion: reduce` 时关闭，不依赖动画才能读到内容。SVG 动效的播放取决于 GitHub 和浏览器，不能承诺所有客户端一致。
- `profile-terminal-*.svg` 以最初 README 的 `fastfetch` 为视觉参考，记录 NixOS / Garuda Linux、Rust / Python、Zed / Helix；`profile-focus-*.svg` 展示系统与内核、实时推理与视觉、AI 安全。个人资料修改时同时更新卡片文字、`title`/`desc` 与 README 的替代文本。
- 两张卡片使用固定像素宽度和透明画布留白，不用 Markdown 表格强制双列；窄屏依靠 GitHub 图片宽度限制自然换行。验收必须检查完整 GitHub 样式中的桌面、390px 和 320px 布局，两种主题、资源加载与减少动态效果，不能只检查图片文件或无样式 Markdown。
- `profile-tools-*.svg` 是 2026-10-03 从 [Skill Icons](https://github.com/tandpfun/skill-icons) 的 `rust,python,linux,nix` 两色输出保存的本地图标，`metadata` 内保留来源和完整 MIT 许可。渲染不请求第三方图标或统计服务；更新资源时须重新核对来源、许可和 SVG 的自包含性，不接受脚本、事件处理器、外部引用或 `foreignObject`。
- 社区参考为 [Orhun](https://github.com/orhun/orhun) 的终端主题、[DenverCoder1](https://github.com/DenverCoder1/DenverCoder1) 的图标分组、[Anurag Hazra](https://github.com/anuraghazra/anuraghazra) 的视觉层次与 [Sindre Sorhus](https://github.com/sindresorhus/sindresorhus) 的个人符号表达；另参考本仓库最初的个人 README。仅借鉴构图方法，不复制他人的履历、动图或统计数据。

## 项目内工具链与启动

需要系统已有的 `rustup` 命令、可用网络及本机 Rust/C++ 编译依赖。`./dev.sh` 将 Rust 1.99.0、WASM 目标、cargo-leptos 0.3.10、wasm-bindgen-cli 0.2.129、wasm-opt 0.116.1、Cargo 注册表、编译产物与临时文件安装或生成在仓库的 `run/`；不安装 Bun/Node，不重装或移动系统工具。wasm-opt 对应 Binaryen 116，以 `--no-default-features` 排除 release WASM 不需要的可选 LLVM/DWARF 调试信息支持，仍执行真实优化；不能用跳过优化器代替 release 构建。首次准备时间和空间取决于网络及本机编译环境。

`Cargo.toml` 的开发配置使用 `debug = 1` 保留有限调试信息和行号；完整调试信息曾使 Leptos 单态化产物超过链接器 4 GiB 调试节限制并报 `R_X86_64_32 out of range`。这不改变发布配置，但开发调试不再包含完整变量/类型信息；不要用关闭 hydration 一致性设置来规避链接问题。

```sh
./dev.sh setup
./dev.sh leptos build
./dev.sh run --locked
```

打开 `http://127.0.0.1:3000`。换端口：`BLOG_ADDR=127.0.0.1:4000 ./dev.sh run --locked`；开发时可运行 `./dev.sh leptos watch`。服务默认仅监听回环地址；不要在未配置 TLS 和部署安全边界前公开暴露。停止服务使用 `Ctrl+C`。

**本项目所有 Cargo 与 Rustup 命令均经 `./dev.sh` 执行**，例如 `./dev.sh test --locked`、`./dev.sh fmt -- --check`、`./dev.sh clippy --all-targets -- -D warnings`、`./dev.sh -- rustc --version`。脚本隔离 `CARGO_HOME`、`RUSTUP_HOME`、`CARGO_TARGET_DIR`、XDG 数据/缓存/配置、`TMPDIR`、Trivy 缓存、Leptos 输出和构建器工具安装目录到 `run/`；直接执行 `cargo`/`rustup` 或外部浏览器不受约束，可能写入用户目录。`run/` 和 `.env` 已忽略，不会提交编译产物和本地设置。其他工具可用 `./dev.sh -- <命令>` 继承隔离变量。

### 静态构建与发布

准备工具链后运行 `./export.sh`。脚本使用独立的 `run/site-release/` 构建 release SSR/WASM，再直接调用同一次 cargo-leptos 构建的 `run/build/release/mcb-smart-boy export`，输出完整站点到 `run/pages/`，不监听 HTTP 端口。开发构建继续使用 `run/site/`，两套资源不混用。不要另用不同配置重建导出二进制，否则可能破坏 hydration 标记一致性。

导出页面、阅读树、面包屑和 Markdown 内容链接共用生成索引；不手写导出网址表。每个规范网址写成目录内 `index.html`，未知网址使用真实错误页 `404.html`，不是首页副本或 JS 跳转。`src/app.rs` 的单一 `SiteContent` 分发器使未知根路径和未知 `/writing/...` 使用相同错误页面结构；本地 SSR 通过官方 `LeptosRoutes` 注册索引网址，以保留 executor 初始化、请求和 meta 上下文。

CSS、JS、WASM 使用构建生成的哈希文件名，HTML 引用与最终资源一致；发布包不包含 AutoReload、开发 WebSocket 或服务器二进制。导出检查必需的哈希资源、路径冲突、符号／硬链接和文件归属，在临时目录组装成功后才替换 `run/pages/`。已有输出必须与 `.mcb-static-export` 清单完全匹配；不要手工往产物目录加入文件。准备失败保留旧输出，替换失败尝试恢复；安装和恢复都失败时保留备份并报告路径与两个原因，不能宣称所有文件系统故障都可自动回滚。构建输入视作可信本地源码，链接检查不是针对同权限恶意并发改写的安全沙箱。

字体、图片及完整许可原样保留。本轮发布目录约 93 MiB，其中字体约 90 MiB；本地未压缩静态服务的中文首页首次资源请求约 43.5 MiB，其中 WASM 约 1.83 MiB。这不是线上压缩传输量或性能基准。完整字体仍是主要下载负担，本轮未擅自裁剪字形或更换字体。

`.omp/pages/workflow.yml` 提供独立 Pages 仓库 `MCB-SMART-BOY/mcb-smart-boy.github.io` 使用的 `.github/workflows/pages.yml` 配置，不在源码仓库自动部署。网站地址为 `https://mcb-smart-boy.github.io/`，Pages source 使用 GitHub Actions。首次安装前须确认目标仓库用途，只在获得对应授权后创建或更新，不覆盖已有站点或未识别内容。

发布只接受手动提供的完整、小写 40 位源码 commit SHA，从 `MCB-SMART-BOY/MCB-SMART-BOY` 检出并核对实际 SHA，运行锁定工具链和项目验证器，上传同一 run 的 Pages artifact 后部署。源码 SHA 另存为 provenance artifact，不混入网站。默认 `contents: read`，仅 deploy job 增加 `pages: write` / `id-token: write`；Action 固定完整提交，不保存 checkout 凭据，不新增跨仓库 PAT 或 push 自动发布。全局质量门仍是本地独立要求，不把项目 validator 冒称为所有安全检查。

提交、推送、创建仓库、部署和回滚均需对应授权。首次上线没有旧版可恢复；发布失败时停止后续发布并保留诊断，不虚构回滚基线。后续恢复旧版前先核对旧 run 的已验收 artifact 是否仍可用；产物过期时按记录的旧 SHA 重新构建、验收后再发布。每次发布后必须实际验证 HTTPS、深链刷新、无尾斜线请求、真实 404、WASM MIME、资源完整性，并记录已验收的源码 SHA 和 workflow run；本地静态服务通过不能替代 Pages 平台验收。线上回滚未经演练时必须明确说明。

## 验证与维护

1. 先以纯静态服务提供 `run/pages/`，验证首页、全部目录和文章、深层锚点、query/hash、刷新、404、历史导航与原有交互；覆盖两种语言／主题、手机、无 JavaScript、WASM 失败／延迟及异常 cookie，再运行项目验证器 `./validate.sh`（WASM typecheck、SSR/WASM 构建、Rust 测试、release 静态导出）。改动发布配置时另做 actionlint 与 ShellCheck。
2. 最后运行通用质量门：`./dev.sh -- "$HOME/.omp/agent/validate.sh"`；其与项目 validator 相互独立。适用 gate 无法通过时明确记录原因，不把跳过当作通过。
3. 除非用户明确授权，不提交、推送、部署或改动个人主页中已核实的资料。用户明确同意的依赖/SSR 迁移取代旧 Askama 模板；不保留旧端点或重复渲染路径。

## 页面与内容

| 路径 | 内容 |
|---|---|
| `/`、`/#home` | 单页文档 / 首屏简介；主按钮仅保留阅读文章和了解我，不重复展示精选项目摘要 |
| `/writing/` | 内容根 `_index.md` 的介绍与实际直接子项，导出为 `writing/index.html` |
| `/writing/*content_path/` | 按真实内容路径精确分发目录或文章，并导出相应 `index.html`；源码路由拒绝未知路径、无尾斜线和重复斜线 |
| 未知路径 | 静态托管使用根 `404.html` 并保留 HTTP 404；不是客户端假成功页 |
| `/#about` | 同页关于简介，与后续关注分区及资料章节独立呈现 |
| `/#focus` | 同页关注分区：系统与开发工具、形式化验证、AI 与工程实践；链接对应项目或经历 |
| `/#open-source` | 项目与开源：现有 5 个公开项目，不恢复首页“精选项目”按钮 |
| `/#experience` | 工程经历：现有 3 条脱敏工程与研究经历 |
| `/#background` | 教育与技能：软件工程学习背景、语言与后端、系统环境、验证与 AI |
| `/#codeflow` | CodeFlow 社团介绍 |
| `/#recognition` | 荣誉与交流：现有 4 项荣誉和 1 项交流 |
| `/#notes` | 示例文章：单页中的文章预览；完整内容目录仍由底部归档入口进入 |

博客简介和项目链接以 `src/profile.rs` 为准，固定资料随 `Locale` 切换，中英文不补造事实。项目使用当前仓库名 `n3v3`，关于分区仅说明一次“原 Neve”。保留 `/#experience`、`/#codeflow` 真实资料锚点；删除“精选项目”按钮、`#projects` 快捷入口及对应 id，项目事实与 `projects-title` 标题仍保留。卡片随正文容器宽度在三列、两列、单列之间切换。旧 `/focus`、`/about` 返回 404，不维护别名或重定向。仅改变 README 展示不是源码隐私隔离；将博客源码推送到公开仓库仍会公开其中的资料，提交和发布须另行授权。

主导航按正文顺序提供八个锚点：首页、关于、关注领域、项目与开源、工程经历、教育与技能、CodeFlow 社团、示例文章。荣誉与交流不再出现在主导航中，正文和 `/#recognition` 深链仍保留在社团与示例文章之间。`LandingSection::ALL` 登记完整九节的顺序、路径和目标 ID，`iter_navigation()` 仅过滤荣誉项，桌面展开、折叠图标、移动抽屉与无 JavaScript 后备导航共用该迭代器；滚动采样和面包屑仍覆盖全部九节，处于荣誉节时不将其他主导航项冒充为当前节。侧栏和面包屑使用同一活动状态，长英文标签完整换行。个人资料的片段链接统一使用普通 `<a target="_self">`，显式绕过当前 Router 的点击拦截。同文档由浏览器原生更新 hash、定位和处理键盘焦点；从阅读页进入时先加载完整 HTML 文档再定位。当前 Leptos 客户端曾复现跨页 hash 不定位、同页侧栏锚点焦点停在正文开头，不能恢复无 opt-out 的路径，也不补写 History 掩盖问题。移动菜单点击任一内容锚点后关闭并以 `preventScroll` 聚焦实际目标；阅读树的目录保持菜单打开、文章关闭的规则不变。

首页、关于、关注三个主分区至少一屏，其余资料章节保留自然高度；正文不拦截滚轮、不强制吸附或内部滚动。九个外层 `.landing-section` 保持稳定的锚点几何，仅直接子 `.landing-motion` 随滚动逐节过渡：进场上移渐显，离场缩小淡出，反向滚动会恢复。长章节的中段保持完全清晰，尾部接近离场时才开始缩退。默认内容可见；无 JavaScript、动效偏好无法读取或启用减少动态效果时保持静态，键盘可见焦点所在节也不淡化。参考 [Codrops 分区滚动过渡](https://tympanus.net/codrops/2024/01/31/on-scroll-animation-ideas-for-sticky-sections/) 的缩退与接替构图，不引入 GSAP，也不再依赖 CSS scroll timeline。

`src/landing.rs` 缓存九个定位目标及其内部动效节点，合并 scroll/resize/hashchange/popstate、尺寸观察和系统动态偏好变更到可取消帧；每帧先读取全部外层几何，再写内部视觉属性，不额外建立动画循环。`src/landing_motion.rs` 计算各节独立的进出进度，并复用数值文本缓冲区；离开根路由时清理监听、尺寸观察和待执行帧。高亮激活线为顶栏下方 16px 加 1px 取整容差，与 CSS `scroll-margin-top` 对齐，避免短章节点击后提前高亮下一项；实际滚动到底时选最后一项。普通滚动只更新活动分区、面包屑和动效，不改 URL、历史或焦点。SSR 初值固定首页，hydrate 后再按真实布局更新，避免初始 hash 造成标记分叉。

新增内容只维护 `content/posts/`：目录提供 `_index.md`，文章在 front matter 保存标题、摘要、日期及阅读分钟数，目录与文章的 `weight` 统一控制同级顺序。构建脚本递归发现内容，校验 YAML、路径冲突和相对 Markdown 链接，生成 SSR 与 WASM 共用的 `Directory`、`Post`、`ContentEntry` 索引；`POSTS` 是生成结果，不是手写登记表。每个目录可直接放文章、子目录或两者混合，也可为空；不再有固定 Book/Chapter 模型。具体字段、限制、网址与维护流程见 [内容规范](content-format.md)。

内容正文编入二进制；新增、移动、改标题或改排序后必须重建并重新导出静态站点，`./dev.sh leptos watch` 仅负责本地开发监听。YAML 解析器仅为 build/dev 依赖，完整消费一份文档并限制解析资源，不启用文件包含或属性展开。出错时构建失败并报告文件与原因，不静默遗漏内容。原始 HTML 和危险链接仍不执行；相对 `.md`／`_index.md` 链接检查目标页面并改写为规范网址。生成索引供导出、页面、左树、面包屑、父级入口和首页列表共同使用，目录介绍只显示本目录正文，不拼接子目录介绍。

所有非根内容网址统一以 `/` 结尾，并严格随文件位置变化，不维护 slug、permalink、别名或重定向表。`content/posts/demo-notes/systems/small-systems.md` 对应 `/writing/demo-notes/systems/small-systems/`；旧文章地址、`/writing/books/...` 和旧章节地址不再解析。Markdown 物理来源路径与公开 URL 分开处理。应用路由拒绝无尾斜线等非规范路径；静态托管平台可能先做目录重定向，其真实状态码须上线后核验，不能把本地预览的 301 当成 Pages 保证。HTML 首次响应即包含可阅读正文，hydration 后客户端导航与偏好交互可用。

阅读区采用稳定的双栏分工：左栏提供真实目录与文章的可展开内容树；右栏仅在文章有真实 Markdown 小标题时显示“本文目录”，不重复文章标题。根目录、其他目录及无效路由不显示右目录，也不在正文重复插入另一份导航树。左栏顶部固定返回主导航、阅读导航标题和根目录入口；当前页精确标记，深链自动展开真实祖先。根下默认展开排序后的首个目录，其他手动展开状态保留到组件销毁。树标题完整换行，行高至少 44px，深层缩进最多累计到三级以保留窄栏可读宽度。

保留主导航与阅读区两个纵向面板，不再按内容层级横向换页。主导航向下滚轮先滚动当前长列表，到底后才进入 `/writing/`；只有位于 `/writing/` 且左树已滚到顶部时，向上滚轮才返回首页，目录和文章深页只滚动而不跳出阅读。沿用惯性抑制和键盘切换，不截获目录链接的正常键盘操作。主导航文章入口位于面板底部，短视口可随面板滚动到达；折叠后的八项图标与归档入口也置于独立纵向滚动容器，滚轮不穿透到正文。折叠栏归档入口先展开侧栏再进入内容根，不重复写入历史。短视口中阅读头部固定、当前面板内部滚动。

文章目录随正文区域而非整个窗口响应：`.site-workspace` 至少 1100px 时显示右栏，否则使用正文原生 `details.article-outline-compact`；没有目录的页面不留空列。拖动或折叠左栏会重新分配宽度。`site.css` 断点须与 `directory_focus.rs` 的 `CURRENT_DIRECTORY_MIN_WORKSPACE_PX` 保持一致。目录焦点迁移合并到可取消的动画帧，执行时根据实际可见目录选择 summary 或右栏标题，不打开折叠内容、不滚动页面；快速反向跨断点不会执行旧目标，用户主动改换焦点会取消迁移，路由切换的正文焦点请求优先。

树使用原生 `nav`、`ul`、`li`。目录整行是真实页面链接：普通点击导航并展开；再次点击当前目录只切换展开，不新增历史。箭头是装饰，不是另一枚按钮；链接持有唯一的桌面／移动端 `aria-controls` 和 `aria-expanded`。SSR 展开全部子列表，hydration 后才显示箭头并折叠非活动分支；无 JavaScript 时所有真实链接仍可访问。Alt/Ctrl/Meta/Shift 修饰操作保持浏览器原义，隐藏面板设为 `inert`。移动抽屉中，目录链接、根目录入口及主导航／阅读区切换保持抽屉打开；文章和段落链接关闭抽屉并聚焦正文，关闭按钮或 Escape 返回菜单按钮。语言、主题和侧栏折叠不重置树的展开状态。

两个文章目录共用由 `SiteShell` 提供的当前标题状态。标题节点按文章缓存，滚动、窗口缩放和历史锚点事件合并为可取消的动画帧采样；换文章清理旧监听和节点。普通滚动只更新 `aria-current="location"`，不改 URL、历史或焦点，点击仍使用真实锚点。标题定位允许 1px 取整误差，实际滚动到底时选中末标题，不能以严格坐标相等或仅靠 hash 判断当前阅读位置。

两种主题共用网格、圆角、间距、装饰和响应式布局，仅通过色彩变量区分浅色暖纸与夜间紫黑。首页标题按实际文案语言和正文可用宽度缩放，英文保持两行；中等宽度改为单列，不为英文维护第二份页面。侧栏支持折叠、拖动宽度和移动端抽屉。字体由 `assets/site.css` 的 `@font-face` 自托管 Maple Mono NF、JetBrainsMono Nerd Font Mono、思源黑体与思源宋体 Regular/Bold；微软雅黑与仿宋仅为访客系统字体回退，不重新分发。`public/fonts/NOTICE.txt` 记录来源、SHA 与许可证路径；完整 CJK OTF 较大，在调整字体打包方案时核对字形覆盖与许可。无后台、登录、评论、在线编辑或 RSS；Pages 发布配置与授权边界见上文。正式发布前需替换示例文章、审阅仓库内容，并评估部署安全、浏览器兼容性和可访问性。

## 博客设计来源

调研了 [Matklad](https://matklad.github.io/) 的单栏文章、[Anthony Fu](https://antfu.me/posts) 的列表密度、[Dan Luu](https://danluu.com/) 的长文归档、[Tarik Karahodžić](https://www.tarikkarahodzic.dev/projects/personal-site) 的留白，以及 [Stefan Vitasović](https://tympanus.net/codrops/2025/03/05/case-study-stefan-vitasovic-portfolio-2025/) 的几何构图。浅色与文章背景借鉴本账号 [resume-template](https://github.com/MCB-SMART-BOY/resume-template) 中 `assets/styles/resume.css` 的暖纸底色、20px 浅网格、靛蓝点缀和正文深灰，而非复制用于 A4 导出的整张背景图。博客以本地 CSS 构图、大字和清晰的文章层级呈现，不复制第三方素材或加入重型动画，尊重系统减少动效偏好。

阅读导航借鉴 [Docusaurus shared sidebar](https://docusaurus.io/docs/sidebar) 的稳定内容树和独立页内目录、[mdBook SUMMARY](https://rust-lang.github.io/mdBook/format/summary.html) 的内容层级，以及 [W3C APG Disclosure Navigation](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/examples/disclosure-navigation/) 的普通导航语义。按用户选择采用目录整行导航／展开，而非复制示例中的独立展开按钮；保留本站主题、字体、Logo 动效及主导航／阅读区滚轮切换。文件内容规范借鉴 Hugo 和 Zola 的 `_index.md` 约定，参考链接见 [content-format.md](content-format.md)。
