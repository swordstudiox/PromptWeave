# PromptWeave 织语

PromptWeave 是一个面向图像创作者的本地提示词工作台。它可以从 GitHub 上的 GPT-Image / 图像生成提示词合集导入参考模板，帮助用户把自然语言想法整理成更适合图像生成工具使用的 prompt，并支持复制导出、图片预览和历史复用。

项目采用 Tauri 2 + React + TypeScript + Rust + SQLite 构建，运行数据默认保存在当前目录的 `.promptweave/` 中。没有 API Key 时也可以离线使用本地模板库、规则优化和导出功能；配置 API 后可以接入 LLM 提示词优化和图片生成。

> 当前项目处于早期可用阶段，适合试用、二次开发和功能共建。

## 功能特性

- GitHub 提示词库导入：支持 GitHub repo、tree、blob、raw 链接。
- Markdown / JSON 解析：自动提取标题、分类、prompt、negative prompt、比例、标签等信息。
- 参考库来源管理：导入成功后保存来源，可在导入页手动同步。
- 本地模板库：使用 SQLite 保存模板，支持搜索、收藏、编辑、归档。
- 离线提示词优化：不配置 API 也能用本地规则生成中文优化版和英文导出版。
- LLM 提示词优化：支持 OpenAI、Claude、OpenAI-compatible API。
- 图片生成预览：支持 OpenAI-style 图片生成接口，生成结果保存到本地历史目录。
- 多格式导出：支持 GPT Image、Midjourney、Stable Diffusion / ComfyUI。
- 创作参数：支持比例、尺寸、质量、数量、Midjourney stylize/chaos、SD steps/CFG/sampler/seed。
- 历史记录：保存输入、输出、导出格式、参数、匹配模板和生成图片，可载回创作页继续修改。
- 轻量本地工作区：不依赖数据库服务、Python 服务或后台常驻进程。

## 适合谁使用

- 经常使用 GPT Image、Midjourney、Stable Diffusion、ComfyUI 等工具的创作者。
- 想把 GitHub 上的高质量 prompt 合集整理成本地素材库的用户。
- 需要中文输入、英文导出、多格式提示词输出的用户。
- 想基于 Tauri/Rust/React 开发本地图像创作工具的开发者。

## 当前状态

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| 桌面应用 | 已实现 | Tauri 2 + React + Rust。 |
| 本地工作区 | 已实现 | 当前目录 `.promptweave/` 保存数据库、配置、历史和图片。 |
| GitHub 导入 | 已实现 | 支持 repo/tree/blob/raw，导入过程已放到后台线程。 |
| Markdown / JSON 解析 | 已实现 | 支持常见 prompt、negative prompt、aspect ratio、tags 结构。 |
| 来源管理 | 已实现 | 可查看已保存来源并手动同步。 |
| 模板库 | 已实现 | 支持搜索、收藏、编辑、归档。 |
| 提示词优化 | 已实现 | 本地规则 + 可选 LLM API。 |
| 图片生成 | 已实现 | OpenAI-style 图片生成接口。 |
| 历史记录 | 已实现 | 可保存并载回创作页。 |
| 定时同步 | 未实现 | 当前仅支持手动同步。 |
| 向量语义检索 | 未实现 | 当前使用 SQLite FTS 关键词检索。 |
| 模板批量翻译 | 未实现 | 当前只在输出层提供中文优化和英文导出。 |

## 界面模块

PromptWeave 当前包含 5 个主要页面：

- 创作：输入想法，匹配参考模板，优化提示词，选择导出格式，可选生成图片。
- 模板库：搜索、收藏、编辑、归档本地模板。
- 历史：查看创作记录，把历史输入载回创作页。
- 导入：粘贴 GitHub 链接，预览导入，管理来源并手动同步。
- 设置：配置提示词优化 API 和图片生成 API。

## 技术架构

```text
PromptWeave
├─ src/                         # React / TypeScript 前端
│  ├─ components/                # 页面组件
│  ├─ lib/                       # 本地优化、导出格式等前端逻辑
│  └─ types/                     # 前端类型
│
├─ src-tauri/                    # Tauri / Rust 后端
│  ├─ src/
│  │  ├─ main.rs                 # Tauri 命令注册
│  │  ├─ imports.rs              # GitHub 导入与 Markdown/JSON 解析
│  │  ├─ db.rs                   # SQLite schema 和查询
│  │  ├─ config.rs               # 本地配置
│  │  ├─ prompt_api.rs           # 提示词优化 API
│  │  ├─ generation.rs           # 图片生成 API
│  │  └─ workspace.rs            # 本地工作区
│  └─ tauri.conf.json            # Tauri 配置
│
├─ docs/                         # 设计和实现计划
├─ package.json
├─ pnpm-lock.yaml
├─ rust-toolchain.toml
└─ README.md
```

## 本地数据

运行时数据保存在当前启动目录的 `.promptweave/` 中：

```text
.promptweave/
├─ db.sqlite                     # 模板库、来源、历史记录
├─ config.json                   # API 配置
├─ cache/
├─ imports/
├─ exports/
└─ history/images/               # 图片生成结果
```

`.promptweave/` 已在 `.gitignore` 中忽略。不要把它提交到公开仓库。

注意：当前 API Key 以明文保存在 `.promptweave/config.json`。如果你要发布或分享工作区，请先删除该文件或移除敏感字段。

## 环境要求

开发和编译需要：

- Node.js 20 LTS 或更高版本。
- Corepack。
- pnpm 10。
- Rust stable。
- Tauri 2 所需系统依赖。
- Windows 编译需要 Microsoft C++ Build Tools / Visual Studio Build Tools 和 WebView2 Runtime。

检查环境：

```bash
node -v
corepack --version
corepack pnpm -v
rustc -V
cargo -V
```

安装 Rust：

```powershell
winget install Rustlang.Rustup
```

也可以使用 rustup 官方安装方式：https://rustup.rs

## 安装依赖

```bash
corepack enable
corepack pnpm install
```

如果本机已有 pnpm 缓存，也可以离线安装：

```bash
corepack pnpm install --offline
```

## 开发运行

```bash
corepack pnpm dev
```

该命令会启动 Vite 开发服务器，并打开 Tauri 桌面窗口。

## 常用命令

前端测试：

```bash
corepack pnpm test
```

TypeScript 检查：

```bash
corepack pnpm typecheck
```

前端生产构建：

```bash
corepack pnpm vite build
```

Rust 测试：

```bash
cd src-tauri
cargo test
```

Rust 编译检查：

```bash
cd src-tauri
cargo check
```

## 构建可执行文件

### Windows 免安装 exe

```bash
corepack pnpm tauri build --no-bundle
```

输出位置：

```text
src-tauri/target/release/promptweave.exe
```

### Windows 安装包

```bash
corepack pnpm tauri build
```

输出位置通常在：

```text
src-tauri/target/release/bundle/
```

### macOS

在 macOS 上执行：

```bash
corepack pnpm install
corepack pnpm tauri build
```

输出位置通常在：

```text
src-tauri/target/release/bundle/
```

macOS 签名和公证暂未配置。

### Linux

在目标 Linux 环境安装 Tauri 所需系统依赖后执行：

```bash
corepack pnpm install
corepack pnpm tauri build
```

输出位置通常在：

```text
src-tauri/target/release/bundle/
```

Linux 的 deb、rpm、AppImage 等产物取决于 Tauri 配置和本机环境。

### 跨平台构建说明

Tauri 应用建议在目标平台本机构建。也就是 Windows 构建 Windows 产物，macOS 构建 macOS 产物，Linux 构建 Linux 产物。跨平台构建需要额外处理 WebView、系统库、签名和打包工具链，本项目暂不提供默认配置。

## 推荐验证流程

发布前建议执行：

```bash
corepack pnpm test
corepack pnpm typecheck
corepack pnpm vite build
cd src-tauri
cargo test
cargo check
cd ..
corepack pnpm tauri build --no-bundle
```

当前代码库最近一次验证：

- Rust 单元测试：22 个测试通过。
- 前端 Vitest：3 个测试通过。
- TypeScript 类型检查通过。
- Vite 生产构建通过。
- Tauri `--no-bundle` release exe 构建通过。

## API 配置

设置页包含两套独立配置：

### 提示词优化 API

支持：

- OpenAI chat completions
- Claude messages
- OpenAI-compatible chat completions

字段：

- Provider
- Base URL
- Model
- API Key

未启用时，PromptWeave 使用本地规则优化。

### 图片生成 API

支持 OpenAI-style 图片生成接口。接口需要返回 `b64_json` 或图片 `url`。

字段：

- Provider
- Base URL
- Model
- API Key

未启用时，创作页仍可优化和导出提示词，但不能直接生成图片预览。

## 使用流程

### 1. 导入参考库

1. 打开“导入”页。
2. 粘贴 GitHub 仓库、目录、blob 或 raw 链接。
3. 点击“预览导入”。
4. 确认解析结果后点击“导入到本地库”。
5. 后续可在来源列表中点击“同步”重新抓取。

### 2. 管理模板

1. 打开“模板库”页。
2. 搜索模板。
3. 收藏常用模板。
4. 编辑标题、分类、提示词、负面提示词、比例和标签。
5. 归档不再使用的模板。

### 3. 生成提示词

1. 打开“创作”页。
2. 输入中文描述。
3. 系统匹配本地模板并生成优化结果。
4. 选择 GPT Image、Midjourney 或 Stable Diffusion 导出格式。
5. 复制提示词到其他工具，或配置 API 后直接生成图片。

### 4. 复用历史

1. 打开“历史”页。
2. 查看之前的输入、输出和生成图片。
3. 点击“载入到创作页”继续修改。

## 路线图

- 启动时自动检查参考库更新。
- 参考库定时同步。
- embedding 语义检索。
- 模板翻译和本地化缓存。
- 导出为 `.txt`、`.json`、ComfyUI 片段等文件格式。
- 更完整的图片生成服务适配器。
- GitHub tree 链接的目录路径和分支精细识别。
- 更完善的错误提示和导入报告。

## 已知限制

- 当前搜索基于 SQLite FTS，不是向量语义检索。
- 当前同步是手动同步，不会自动定时抓取。
- GitHub tree 链接的目录路径处理还不完整。
- OpenAI-compatible 图片接口如果响应格式不同，需要额外适配。
- API Key 目前明文保存在本地配置文件中。

## 贡献

欢迎提交 Issue 和 Pull Request。建议贡献前先执行完整验证流程，确保前端、Rust 和 Tauri 构建都能通过。

适合优先贡献的方向：

- 更多 prompt 合集解析规则。
- 更好的中文提示词优化规则。
- 向量检索或轻量本地语义检索。
- 更多图片生成 API 适配。
- UI 体验和错误提示改进。
- 文档、截图和示例项目。

## 开源协议

当前仓库尚未添加 LICENSE 文件。正式发布到 GitHub 前，请先选择并添加开源协议，例如 MIT、Apache-2.0 或 GPL 系列。
