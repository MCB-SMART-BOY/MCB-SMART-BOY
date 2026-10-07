# 文件夹与 Markdown 内容规范

## 唯一事实源

`content/posts/` 中的真实位置决定目录、归属和网址；Markdown 保存标题、日期、摘要、排序及正文。`build.rs` 在构建时扫描并校验内容，生成完整 HTML 导出、SSR 开发服务与 WASM 共用的静态索引。`src/content.rs` 只定义通用类型和查询，不登记具体文章；静态导出清单也来自同一索引。

- 每个目录，包括内容根，都必须有 `_index.md`。它描述当前目录，不列出子项。
- 其他小写 `.md` 文件是文章。目录可以直接包含文章、子目录或两者混合，也可以为空；没有固定的“书籍／章节”层数。
- 普通非 Markdown 文件不进入索引，也不会自动作为附件发布。图片等静态资源放在 `public/`，以站点根路径引用。
- 文件及目录名使用 URL 安全的 ASCII 字母、数字、`-`、`_`、`.`；不能以 `.` 开头或结尾。推荐稳定的 kebab-case；显示标题可以用中文。

当前三篇示例文章的位置：

```text
content/posts/
├── _index.md
└── demo-notes/
    ├── _index.md
    ├── systems/
    │   ├── _index.md
    │   └── small-systems.md
    ├── rust-web/
    │   ├── _index.md
    │   └── rust-web-notes.md
    └── learning/
        ├── _index.md
        └── notes-on-learning.md
```

这只是当前内容布局，不是程序预设的三层模型。

## 元数据与正文

文件从独占一行的 `---` 开始，第二条独占一行的 `---` 结束 YAML front matter；其后是普通 Markdown 正文。支持 UTF-8 和 LF／CRLF。元数据不进入正文；页面标题取自 `title`，正文不必再写一个重复的 H1。

目录 `_index.md`：

```markdown
---
title: 探索笔记
weight: 10
---

这里写当前目录的介绍。子项由文件系统生成，不在这里重复登记。
```

文章：

```markdown
---
title: 把系统做小，是一种工程能力
date: "2026-10-03"
summary: 从问题本身出发，去掉不需要的层，给复杂度一个进入系统的理由。
category: SYSTEMS
reading_minutes: 3
weight: 10
---

这里开始写正文。

## 一个具体问题

正文中的小标题会生成文章目录和段落锚点。
```

| 字段 | 目录 | 文章 | 约束 |
|---|---|---|---|
| `title` | 必填 | 必填 | 非空文本 |
| `weight` | 可选 | 可选 | 非负整数，最大 4294967295；省略时排在显式权重之后 |
| `date` | 不支持 | 必填 | 有效的 `YYYY-MM-DD` 日历日期，建议加引号 |
| `summary` | 不支持 | 必填 | 非空文本 |
| `reading_minutes` | 不支持 | 必填 | 1–255 的整数 |
| `category` | 不支持 | 可选 | 提供时不能是空白文本；仅显示，不决定目录归属 |

同一目录下所有直接子项一起按 `weight` 升序排列，同权重或都未设置时按内容路径稳定排序。首页文章按日期倒序排列，同日按路径排序。显示日期和序号由程序派生，不重复维护。

未知字段、重复字段、非法值、YAML 锚点／别名、合并键和不支持的标签会使构建失败。解析必须完整消费一份有效文档，不接受文档结束标记后的坏 YAML。为限制误输入，单文件不超过 2 MiB、front matter 不超过 16 KiB，目录最多向下嵌套 32 层，整个索引最多 10000 个目录及文章。不允许符号链接，发现即构建失败。

界面语言切换只翻译固定 UI。文章及目录的原文标题、摘要和正文不伪装成英文翻译。

## 网址与链接

- 内容根 → `/writing/` → `writing/index.html`。
- `demo-notes/systems/_index.md` → `/writing/demo-notes/systems/` → 相应目录 `index.html`。
- `demo-notes/systems/small-systems.md` → `/writing/demo-notes/systems/small-systems/` → 相应目录 `index.html`。
- 文章名只去掉最后一个 `.md` 后缀，所有非根页面的公开网址统一以 `/` 结尾；Markdown 来源文件路径本身不追加斜线。
- 同一位置的 `name.md` 与 `name/_index.md` 会产生网址冲突，构建必须失败。
- 不支持 `slug`、`permalink`、别名或重定向表。移动或重命名内容就会改变网址。

正文推荐使用相对 Markdown 链接，如 `[同目录文章](other.md)`、`[父目录](../_index.md)`、`[段落](other.md#section-1)`。构建检查目标页面存在，并禁止越过内容根；渲染时改写为对应页面的规范网址，保留 `#` 片段。该检查不验证片段是否对应某个小标题，也不探测外部网站或一般根路径链接的可达性。相对内容链接不支持查询参数、百分号编码或反斜线。

例如 `other.md#section-1` 会生成同目录的 `other/#section-1` 规范页面链接，而不是寻找名为 `other.md/` 的文件。目录介绍、面包屑、阅读树、父级入口与静态导出共用这些网址。

HTTP(S) 外链可正常指向 `.md` 文件。原始 HTML 只作为文本显示，危险协议不会成为可执行链接；不要把 Markdown 当作脚本入口。

旧的 `/writing/small-systems`、`/writing/books/demo-notes` 及旧章节路径已退出路由，不保留兼容入口。站点生成的目录、面包屑、父级入口和文章链接自动跟随新位置；作者手写的 Markdown 相对链接仍需在移动文件时同步更新。

源码路由只识别规范尾斜线网址；无尾斜线请求在静态托管平台上可能被目录重定向，具体状态码必须以部署后实测为准。未知路径应由托管平台返回根 `404.html` 和 HTTP 404，不把首页当作错误页，也不添加客户端跳转补丁。

## DO / DON'T

- **DO** 新增目录时先添加 `_index.md`，新增文章时补齐其 front matter；内容维护不改 Rust 表或路由表。
- **DO** 用 `weight` 调整同级顺序，用实际文件位置调整归属。
- **DO** 修改后运行 `./export.sh` 重新构建并导出到 `run/pages/`；开发时可用 `./dev.sh leptos watch` 监听内容变化，但 watch 不等于更新发布产物。导出目录受归属清单管理，不手工编辑或添加文件；一次成功替换会移除已删除或移动页面的旧产物。
- **DON'T** 在 `_index.md`、`SUMMARY.md` 或 Rust 中再维护一份 children／articles 列表。
- **DON'T** 用分类标签代替目录关系，或依赖已移除的旧网址。
- **DON'T** 把校验失败当成发布成功。错误应指出文件、操作或字段及底层原因，YAML 解析错误保留原始 cause 链；已运行的旧二进制不会凭空获得新内容。

## VERIFY

- [ ] 新建、移动或修改标题／权重后，实际目录页、左树、面包屑、父级链接与网址一致。
- [ ] 目录可以直放文章、混排和为空；HTML 导出、SSR 与 hydration 使用同一份内容结果。
- [ ] 失效相对链接、缺少元数据或网址冲突会阻止构建，不会静默漏掉文章。
- [ ] 直接访问并刷新新的规范地址成功；旧位置和未知路径返回真实 404。源码路由的无尾斜线拒绝行为与托管平台的重定向行为分开验收；无 JavaScript 或 WASM 加载失败仍可阅读和导航。
- [ ] 行为 smoke 后分别运行 `./validate.sh` 与 `./dev.sh -- "$HOME/.omp/agent/validate.sh"`；缺失或失败的 gate 不写成通过。
- [ ] 新增、移动或删除内容后重新导出，`run/pages/` 中不残留旧页面；核对资源加载、深层锚点及 404，不能仅以 HTML 文件存在判定发布成功。首次部署仍需单独授权，发布配置与步骤见 [项目上下文](AGENTS.md#静态构建与发布)。

参考 [Hugo branch bundle](https://gohugo.io/content-management/page-bundles/)、[Zola section](https://www.getzola.org/documentation/content/section/) 的 `_index.md` 惯例，以及 [mdBook SUMMARY](https://rust-lang.github.io/mdBook/format/summary.html) 的目录组织经验。本站只采用所需约定，不宣称兼容这些生成器的全部字段与语法；不采用另一份总目录清单。
