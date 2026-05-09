# PromptWeave 提示织机

PromptWeave 是一个面向普通创作者的轻量级、离线优先图像提示词工作台。它的目标不是再做一个静态 prompt 案例合集，而是在本地把 GitHub 上的 GPT-Image-2 / 图像生成提示词参考库沉淀为可检索、可编辑、可复用的创作素材，并把“输入想法 -> 优化提示词 -> 导出到其他工具 -> 可选生成预览 -> 保存历史”串成闭环。

当前项目使用 Tauri 2 + React + TypeScript + Rust + SQLite 实现。运行数据默认放在当前工作目录的 `.promptweave/` 下，类似 Python 虚拟环境，每个项目目录可以拥有独立的提示词库、配置、历史记录和图片缓存。

## 产品需求从头梳理

PromptWeave 应该解决的问题是：普通创作者不知道如何写高质量图像生成提示词，也不想手动翻 GitHub prompt 合集。软件应当把公开合集、本地模板、LLM 优化、图片预览和历史复用整合在一个轻量桌面应用里。

理想产品形态：

1. 参考库管理
   - 支持直接粘贴 GitHub 仓库、目录、blob、raw 链接导入提示词合集。
   - 支持 Markdown / JSON 解析。
   - 本地缓存导入结果，并能持续同步来源更新。
   - 支持多个参考库来源管理、手动同步、未来定时同步。

2. 模板库管理
   - 本地保存导入模板。
   - 支持搜索、收藏、编辑、归档。
   - 支持保留来源 URL、模型提示、标签、负面提示词、比例等结构化信息。

3. 提示词优化
   - 用户输入中文自然语言描述。
   - 系统从本地模板库中匹配相关参考模板。
   - 离线时使用本地规则生成中文优化版和英文可用版。
   - 配置提示词优化 API 后，可调用 OpenAI / Claude / OpenAI-compatible API 进一步优化。

4. 导出与复用
   - 至少支持 GPT Image、Midjourney、Stable Diffusion / ComfyUI 三类导出格式。
   - 支持尺寸、比例、质量、数量、MJ stylize/chaos、SD steps/CFG/sampler/seed 等参数。
   - 输出既能直接复制，也能作为其他工具输入。

5. 图片预览/生成
   - 配置图片生成 API 后，在创作页直接生成预览图。
   - 图片保存到本地历史目录。
   - 形成“优化 -> 预览 -> 再调整”的闭环。

6. 多语言与本地化
   - 大量 GitHub 合集是英文，工具需要面向中文用户输出中文解释和中文优化结果。
   - 英文模板应能作为参考结构参与优化。
   - 更完整的模板级翻译/本地化可作为后续模块。

7. 离线优先和低配置
   - 没有 API Key 时也能启动、导入已有本地数据、编辑模板、使用本地规则优化、导出提示词。
   - 不依赖数据库服务、Python 服务或后台常驻进程。
   - 网络能力只在 GitHub 导入、来源同步、外部 LLM/API 图片生成时使用。

## 当前实现情况

| 模块 | 当前状态 | 说明 |
| --- | --- | --- |
| 桌面应用骨架 | 已实现 | Tauri 2 桌面应用，React 前端，Rust 后端。 |
| 本地隔离工作区 | 已实现 | 当前目录下 `.promptweave/` 保存数据库、配置、历史和图片。 |
| GitHub 链接导入 | 已实现 | 支持 GitHub repo/tree/blob/raw 链接，解析 Markdown / JSON。 |
| 导入时前端不卡死 | 已实现 | 预览、导入、来源同步命令已移到后台线程执行。 |
| 参考库来源管理 | 已实现 | 导入成功后保存来源，可在导入页查看并手动同步。 |
| 定时同步 | 未实现 | 当前只有手动同步，尚无启动时自动检查或计划任务。 |
| 本地模板库 | 已实现 | SQLite 保存模板，支持列表和搜索。 |
| 模板收藏/编辑/归档 | 已实现 | 模板库页面支持收藏、只看收藏、编辑字段、归档隐藏。 |
| 检索 | 部分实现 | 当前是 SQLite FTS 关键词检索，不是 embedding 语义检索。 |
| 离线提示词优化 | 已实现 | 本地规则生成中文优化版、英文导出版和结构化字段。 |
| LLM 提示词优化 API | 已实现 | 支持 OpenAI、Claude、OpenAI-compatible 配置。 |
| 图片生成 API | 已实现 | 支持 OpenAI-style 图片生成接口，图片保存到本地历史目录。 |
| 两套 API 配置 | 已实现 | 设置页区分“提示词优化 API”和“图片生成 API”。 |
| 导出格式 | 已实现 | GPT Image、Midjourney、Stable Diffusion / ComfyUI。 |
| 创作参数 | 已实现 | 比例、尺寸、质量、数量、MJ 参数、SD 参数。 |
| 历史记录闭环 | 已实现 | 记录输入、提示词、格式、参数、匹配模板、生成图，可载回创作页。 |
| 模板级翻译模块 | 未实现 | 当前只有输出层中英双语和英文模板参考，未做批量翻译/本地化管理。 |
| API Key 加密存储 | 明确不做 | 当前按需求使用 `.promptweave/config.json` 明文保存。 |
| 插件/模型市场 | 未实现 | 未来可扩展，不属于当前轻量 MVP。 |

## 技术架构

```text
PromptWeave
├─ React / TypeScript 前端
│  ├─ 创作页：输入描述、匹配模板、优化、导出、生成图片
│  ├─ 模板库：搜索、收藏、编辑、归档
│  ├─ 历史页：查看历史、载回创作页
│  ├─ 导入页：GitHub 预览导入、来源管理、手动同步
│  └─ 设置页：两套 API 配置
│
├─ Rust / Tauri 命令层
│  ├─ 工作区初始化
│  ├─ GitHub 抓取与 Markdown/JSON 解析
│  ├─ SQLite 数据读写
│  ├─ 提示词优化 API 调用
│  └─ 图片生成 API 调用与图片保存
│
└─ 本地 .promptweave 工作区
   ├─ db.sqlite
   ├─ config.json
   ├─ cache/
   ├─ imports/
   ├─ exports/
   └─ history/images/
```

### 数据流

1. 用户在“导入”页粘贴 GitHub 链接。
2. Rust 后端在后台线程抓取仓库、目录或 raw 文件。
3. 解析 Markdown / JSON 中的 prompt、negative prompt、比例、标签、图片链接等字段。
4. 写入 SQLite 模板表，并记录参考库来源。
5. 用户在“创作”页输入中文描述。
6. 前端调用本地模板搜索，取匹配模板参与本地规则优化。
7. 如启用提示词优化 API，可继续调用外部 LLM 得到更好的提示词。
8. 用户选择导出格式，复制到 GPT Image、Midjourney、ComfyUI 等工具。
9. 如启用图片生成 API，可直接生成图片并保存历史。

## 项目目录

```text
.
├─ src/                         # React 前端
│  ├─ components/                # 页面组件
│  ├─ lib/                       # 本地优化、导出格式等纯前端逻辑
│  └─ types/                     # 前端类型
├─ src-tauri/                    # Tauri / Rust 后端
│  ├─ src/
│  │  ├─ main.rs                 # Tauri 命令注册
│  │  ├─ imports.rs              # GitHub 导入与解析
│  │  ├─ db.rs                   # SQLite schema 和查询
│  │  ├─ config.rs               # 本地配置
│  │  ├─ prompt_api.rs           # 提示词优化 API
│  │  ├─ generation.rs           # 图片生成 API
│  │  └─ workspace.rs            # 工作区目录
│  └─ tauri.conf.json            # Tauri 应用配置
├─ docs/superpowers/             # 设计和实现计划记录
├─ package.json                  # 前端依赖和脚本
├─ pnpm-lock.yaml                # pnpm 锁文件
├─ rust-toolchain.toml           # Rust stable 工具链声明
└─ README.md
```

## 运行数据目录

PromptWeave 的运行数据位于启动目录下的 `.promptweave/`：

```text
.promptweave/
├─ db.sqlite                     # 模板库、来源、历史记录
├─ config.json                   # API 配置，当前为明文保存
├─ cache/                        # 预留缓存目录
├─ imports/                      # 预留导入元数据目录
├─ exports/                      # 预留导出目录
└─ history/images/               # 图片生成结果
```

如果希望一个项目拥有独立提示词库，就在对应项目目录运行 PromptWeave。删除 `.promptweave/` 会清空该目录下的本地数据。

## 环境要求

开发和编译需要：

- Windows 10/11、macOS 或 Linux。
- Node.js 20 LTS 或更高版本，建议启用 Corepack。
- pnpm 10，项目已在 `package.json` 中声明 `packageManager: pnpm@10.0.0`。
- Rust stable，项目通过 `rust-toolchain.toml` 固定使用 stable，并包含 `rustfmt`、`clippy` 组件。
- Tauri 2 所需系统依赖。
- Windows 编译需要 Microsoft C++ Build Tools / Visual Studio Build Tools，以及 WebView2 Runtime。

终端检查：

```bash
node -v
corepack --version
corepack pnpm -v
rustc -V
cargo -V
```

Windows 如果没有 Rust：

```powershell
winget install Rustlang.Rustup
```

或从 https://rustup.rs 安装。安装后重启终端。

## 安装依赖

```bash
corepack enable
corepack pnpm install
```

如果是在已经有 pnpm 缓存的离线环境中：

```bash
corepack pnpm install --offline
```

## 开发运行

启动 Tauri 开发模式：

```bash
corepack pnpm dev
```

这会先启动 Vite 开发服务器，再启动 Tauri 桌面窗口。

只启动前端构建检查：

```bash
corepack pnpm vite build
```

只检查 TypeScript：

```bash
corepack pnpm typecheck
```

运行前端测试：

```bash
corepack pnpm test
```

运行 Rust 测试：

```bash
cd src-tauri
cargo test
```

运行 Rust 编译检查：

```bash
cd src-tauri
cargo check
```

## 编译可执行文件

### Windows 免安装 exe

如果只需要一个可直接运行的 release exe，不需要安装包：

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

具体安装包类型取决于 Tauri 当前平台和 bundle 配置。

### macOS 应用

在 macOS 机器上执行：

```bash
corepack pnpm install
corepack pnpm tauri build
```

输出位置通常在：

```text
src-tauri/target/release/bundle/
```

macOS 签名、公证、证书配置尚未纳入当前项目流程。

### Linux 应用

在目标 Linux 发行版上安装 Tauri 所需系统依赖后执行：

```bash
corepack pnpm install
corepack pnpm tauri build
```

输出位置通常在：

```text
src-tauri/target/release/bundle/
```

Linux 的 deb、rpm、AppImage 等产物取决于 Tauri 配置和本机环境。

### 关于跨平台编译

Tauri 桌面应用通常建议在目标平台本机编译。Windows 上编译 Windows 产物，macOS 上编译 macOS 产物，Linux 上编译 Linux 产物。跨平台编译涉及 WebView、系统库、签名和打包工具链，当前 README 不把它作为默认路径。

## 推荐验证流程

提交或发布前建议完整执行：

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

当前最近一次功能验证覆盖：

- Rust 单元测试：22 个测试通过。
- 前端 Vitest：3 个测试通过。
- TypeScript 类型检查通过。
- Vite 生产构建通过。
- Tauri `--no-bundle` release exe 构建通过。

## API 配置

设置页有两套独立 API 配置：

1. 提示词优化 API
   - `provider`: `openai`、`claude`、`compatible` 或本地规则。
   - `baseUrl`: 自定义服务地址，可留空使用默认官方地址。
   - `model`: 模型 ID。
   - `apiKey`: API Key。

2. 图片生成 API
   - `provider`: `gpt-image`、`compatible` 或禁用。
   - `baseUrl`: 图片生成接口地址。
   - `model`: 图片模型 ID。
   - `apiKey`: API Key。

配置保存到：

```text
.promptweave/config.json
```

注意：当前按项目需求不做 API Key 加密或系统安全存储，配置文件是明文 JSON。不要把 `.promptweave/` 提交到 Git。

## 常见工作流

### 导入 GitHub 参考库

1. 打开“导入”页。
2. 粘贴 GitHub repo、tree、blob 或 raw 链接。
3. 点击“预览导入”。
4. 确认解析条目后点击“导入到本地库”。
5. 导入成功后，该链接会出现在“已保存参考库”列表中。
6. 后续可点击“同步”手动重新抓取。

### 管理模板

1. 打开“模板库”页。
2. 使用搜索框查找模板。
3. 可收藏、取消收藏、编辑标题/分类/提示词/负面提示词/比例/标签。
4. 不需要的模板可归档，归档后不会出现在模板库和搜索结果中。

### 创作并导出提示词

1. 打开“创作”页。
2. 输入中文自然语言描述。
3. 系统自动匹配本地模板并用本地规则生成优化提示词。
4. 选择导出格式：GPT Image、Midjourney、Stable Diffusion。
5. 根据目标工具调整参数。
6. 复制提示词，或启用 API 后直接生成预览图。

### 查看历史

1. 打开“历史”页。
2. 查看之前复制、API 优化或图片生成时保存的记录。
3. 点击“载入到创作页”复用历史输入和导出格式。

## 当前限制和后续路线

当前限制：

- 检索是 SQLite FTS 关键词检索，不是向量 embedding 语义检索。
- 参考库支持手动同步，尚无定时同步和启动时自动同步。
- GitHub tree 链接目前主要按仓库默认分支递归扫描，目录路径/分支精细同步还可增强。
- 模板级翻译和批量本地化还未实现。
- 图片生成 API 采用 OpenAI-style 图片生成响应解析，非兼容接口可能需要适配。
- API Key 明文保存，这是当前项目明确接受的轻量化取舍。

建议后续优先级：

1. 启动时自动检查参考库更新，或提供轻量定时同步配置。
2. 增加 embedding 语义检索，可选本地小模型或外部 embedding API。
3. 增加模板翻译/本地化缓存，把英文模板结构转为中文创作语境。
4. 增加导出文件功能，把提示词保存为 `.txt`、`.json`、ComfyUI 工作流片段等。
5. 增强 GitHub tree 目录路径识别和同步差异报告。
6. 增加图片生成 API 适配器配置，支持更多服务响应格式。

## 项目名称

当前项目名是 PromptWeave，中文名“提示织机”。这个名字适合当前定位：把用户输入、参考模板、优化模型、导出格式和历史记录编织成一个创作工作流。

备选名：

- PromptWeave：偏产品化，适合长期使用。
- Prompt Loom：和“织机”同义，更偏英文品牌。
- ImagePrompt Studio：直白但普通。
- PromptForge：强调打磨和生成，但偏重开发者气质。

当前建议继续使用 PromptWeave。
