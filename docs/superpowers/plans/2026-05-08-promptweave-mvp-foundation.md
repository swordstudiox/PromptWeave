# PromptWeave MVP Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first runnable PromptWeave foundation: isolated workspace layout, Tauri/React app skeleton, local SQLite schema, prompt import interfaces, local prompt optimization interfaces, and the main creator workspace shell.

**Architecture:** Use Tauri as the desktop shell, React + TypeScript for the UI, and Rust commands for local workspace/database operations. Keep online APIs behind Provider interfaces so the app remains fully usable offline.

**Tech Stack:** Tauri, Rust, React, TypeScript, SQLite, SQLite FTS5, Tailwind/Radix-compatible component structure.

---

## Scope

This plan implements the foundation only. It does not implement every import adapter, every image provider, or a polished production UI. It creates the structure needed to add those features safely.

## File Structure

- Create: `package.json`  
  Frontend package manifest and scripts.
- Create: `pnpm-workspace.yaml`  
  Keeps the JavaScript workspace explicit and isolated.
- Create: `.npmrc`  
  Pins package manager behavior for reproducible local installs.
- Create: `.gitignore`  
  Ignores build output, dependency directories, and workspace runtime data.
- Create: `README.md`  
  Documents setup, isolation model, and first-run behavior.
- Create: `src/main.tsx`  
  React entry point.
- Create: `src/App.tsx`  
  Main desktop shell and navigation.
- Create: `src/styles.css`  
  Base layout styles.
- Create: `src/types/prompt.ts`  
  Shared prompt/template TypeScript types.
- Create: `src/lib/workspace.ts`  
  Frontend wrapper around Tauri workspace commands.
- Create: `src/lib/localOptimizer.ts`  
  Frontend local prompt optimization prototype.
- Create: `src/lib/exportFormats.ts`  
  GPT Image, Midjourney, and Stable Diffusion export formatters.
- Create: `src/components/CreatorWorkspace.tsx`  
  Main creation page.
- Create: `src/components/TemplateLibrary.tsx`  
  Local template library placeholder.
- Create: `src/components/ImportPanel.tsx`  
  GitHub import placeholder and URL normalization preview.
- Create: `src/components/SettingsPanel.tsx`  
  Split prompt API and image API settings UI placeholder.
- Create: `src-tauri/Cargo.toml`  
  Tauri Rust package manifest.
- Create: `src-tauri/tauri.conf.json`  
  Tauri application config.
- Create: `src-tauri/src/main.rs`  
  Tauri command entry point.
- Create: `src-tauri/src/workspace.rs`  
  Workspace isolation and `.promptweave` directory management.
- Create: `src-tauri/src/db.rs`  
  SQLite schema bootstrap.
- Create: `src-tauri/src/imports.rs`  
  GitHub URL classification and import source metadata types.
- Create: `src-tauri/src/providers.rs`  
  Provider configuration structs for separate prompt and image APIs.

## Task 1: Project Metadata And Isolation Defaults

**Files:**
- Create: `package.json`
- Create: `pnpm-workspace.yaml`
- Create: `.npmrc`
- Create: `.gitignore`
- Create: `README.md`

- [ ] **Step 1: Create package manifest**

Create `package.json`:

```json
{
  "name": "promptweave",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "typecheck": "tsc --noEmit",
    "test": "vitest run"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@vitejs/plugin-react": "^5.0.0",
    "vite": "^7.0.0",
    "typescript": "^5.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "vitest": "^3.0.0"
  },
  "packageManager": "pnpm@10.0.0"
}
```

- [ ] **Step 2: Create pnpm workspace config**

Create `pnpm-workspace.yaml`:

```yaml
packages:
  - "."
```

- [ ] **Step 3: Create npm isolation config**

Create `.npmrc`:

```ini
engine-strict=true
auto-install-peers=false
strict-peer-dependencies=false
```

- [ ] **Step 4: Create gitignore**

Create `.gitignore`:

```gitignore
node_modules/
dist/
target/
src-tauri/target/
.promptweave/
*.db
*.sqlite
*.log
.env
.env.*
```

- [ ] **Step 5: Create README**

Create `README.md`:

```markdown
# PromptWeave

PromptWeave is a lightweight offline-first image prompt workspace for creators.

## Isolation model

PromptWeave stores runtime data inside a workspace-local `.promptweave/` directory:

- `db.sqlite` for local prompt templates and history
- `cache/` for imported source files
- `imports/` for GitHub import metadata
- `exports/` for copied/exported prompt files
- `history/` for generated prompt and image records

This keeps each workspace self-contained, similar to a Python virtual environment.

## Development

Use the project-local package manager lockfile and Tauri bundle. Do not require users to install Python, a separate database server, or global app services.

```bash
pnpm install
pnpm dev
```
```

- [ ] **Step 6: Commit**

Run:

```bash
git add package.json pnpm-workspace.yaml .npmrc .gitignore README.md
git commit -m "chore: add project metadata and isolation defaults"
```

Expected: commit succeeds.

## Task 2: Tauri Rust Workspace Bootstrap

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/workspace.rs`
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: Create Rust manifest**

Create `src-tauri/Cargo.toml`:

```toml
[package]
name = "promptweave"
version = "0.1.0"
description = "Lightweight offline image prompt workspace"
edition = "2021"

[lib]
name = "promptweave_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri = { version = "2", features = [] }
thiserror = "2"
rusqlite = { version = "0.32", features = ["bundled"] }
```

- [ ] **Step 2: Create Tauri config**

Create `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "PromptWeave",
  "version": "0.1.0",
  "identifier": "com.promptweave.app",
  "build": {
    "beforeDevCommand": "pnpm vite --host 127.0.0.1",
    "devUrl": "http://127.0.0.1:5173",
    "beforeBuildCommand": "pnpm vite build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "PromptWeave",
        "width": 1280,
        "height": 820,
        "minWidth": 960,
        "minHeight": 680
      }
    ]
  }
}
```

- [ ] **Step 3: Create workspace module**

Create `src-tauri/src/workspace.rs`:

```rust
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct WorkspaceInfo {
    pub root: String,
    pub data_dir: String,
    pub database_path: String,
}

pub fn ensure_workspace(root: &Path) -> Result<WorkspaceInfo, String> {
    let data_dir = root.join(".promptweave");
    let cache_dir = data_dir.join("cache");
    let imports_dir = data_dir.join("imports");
    let exports_dir = data_dir.join("exports");
    let history_dir = data_dir.join("history");

    for dir in [&data_dir, &cache_dir, &imports_dir, &exports_dir, &history_dir] {
        fs::create_dir_all(dir).map_err(|err| format!("Failed to create {}: {err}", dir.display()))?;
    }

    let database_path = data_dir.join("db.sqlite");

    Ok(WorkspaceInfo {
        root: root.display().to_string(),
        data_dir: data_dir.display().to_string(),
        database_path: database_path.display().to_string(),
    })
}

pub fn default_workspace_root() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|err| format!("Failed to resolve current directory: {err}"))
}
```

- [ ] **Step 4: Create database bootstrap**

Create `src-tauri/src/db.rs`:

```rust
use rusqlite::Connection;
use std::path::Path;

pub fn bootstrap(database_path: &Path) -> Result<(), String> {
    let connection = Connection::open(database_path)
        .map_err(|err| format!("Failed to open database {}: {err}", database_path.display()))?;

    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS prompt_templates (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              category TEXT NOT NULL DEFAULT '',
              source_repo TEXT NOT NULL DEFAULT '',
              source_url TEXT NOT NULL DEFAULT '',
              source_license TEXT,
              author TEXT,
              model_hint TEXT NOT NULL DEFAULT 'generic',
              language TEXT NOT NULL DEFAULT 'mixed',
              prompt_original TEXT NOT NULL,
              prompt_zh TEXT,
              prompt_en TEXT,
              negative_prompt TEXT,
              aspect_ratio TEXT,
              tags_json TEXT NOT NULL DEFAULT '[]',
              preview_image_urls_json TEXT NOT NULL DEFAULT '[]',
              imported_at TEXT NOT NULL,
              content_hash TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS prompt_templates_fts USING fts5(
              title,
              category,
              prompt_original,
              prompt_zh,
              prompt_en,
              tags_json,
              content='prompt_templates',
              content_rowid='rowid'
            );

            CREATE TABLE IF NOT EXISTS generation_history (
              id TEXT PRIMARY KEY,
              user_input TEXT NOT NULL,
              prompt_zh TEXT NOT NULL,
              prompt_en TEXT NOT NULL,
              export_format TEXT NOT NULL,
              image_path TEXT,
              created_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(|err| format!("Failed to bootstrap database: {err}"))?;

    Ok(())
}
```

- [ ] **Step 5: Create Tauri command entry point**

Create `src-tauri/src/main.rs`:

```rust
mod db;
mod workspace;

#[tauri::command]
fn init_workspace() -> Result<workspace::WorkspaceInfo, String> {
    let root = workspace::default_workspace_root()?;
    let info = workspace::ensure_workspace(&root)?;
    db::bootstrap(std::path::Path::new(&info.database_path))?;
    Ok(info)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init_workspace])
        .run(tauri::generate_context!())
        .expect("failed to run PromptWeave");
}
```

- [ ] **Step 6: Commit**

Run:

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/src/main.rs src-tauri/src/workspace.rs src-tauri/src/db.rs
git commit -m "feat: add tauri workspace bootstrap"
```

Expected: commit succeeds.

## Task 3: Frontend Shell

**Files:**
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles.css`
- Create: `src/lib/workspace.ts`

- [ ] **Step 1: Create Vite HTML entry**

Create `index.html`:

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PromptWeave</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 2: Create workspace command wrapper**

Create `src/lib/workspace.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceInfo {
  root: string;
  data_dir: string;
  database_path: string;
}

export async function initWorkspace(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("init_workspace");
}
```

- [ ] **Step 3: Create React entry**

Create `src/main.tsx`:

```tsx
import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

- [ ] **Step 4: Create app shell**

Create `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { CreatorWorkspace } from "./components/CreatorWorkspace";
import { ImportPanel } from "./components/ImportPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { TemplateLibrary } from "./components/TemplateLibrary";
import { initWorkspace, type WorkspaceInfo } from "./lib/workspace";

type View = "creator" | "templates" | "history" | "imports" | "settings";

const navItems: Array<{ id: View; label: string }> = [
  { id: "creator", label: "创作" },
  { id: "templates", label: "模板库" },
  { id: "history", label: "历史" },
  { id: "imports", label: "导入" },
  { id: "settings", label: "设置" },
];

export default function App() {
  const [view, setView] = useState<View>("creator");
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWorkspace().then(setWorkspace).catch((err) => setError(String(err)));
  }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <strong>PromptWeave</strong>
          <span>提示织机</span>
        </div>
        <nav>
          {navItems.map((item) => (
            <button
              key={item.id}
              className={item.id === view ? "nav-item active" : "nav-item"}
              onClick={() => setView(item.id)}
            >
              {item.label}
            </button>
          ))}
        </nav>
      </aside>
      <section className="main-panel">
        {error ? <div className="error-banner">{error}</div> : null}
        {workspace ? <div className="workspace-path">工作区：{workspace.data_dir}</div> : null}
        {view === "creator" ? <CreatorWorkspace /> : null}
        {view === "templates" ? <TemplateLibrary /> : null}
        {view === "history" ? <div className="placeholder">历史记录将在本地保存每次输入、输出和生成图。</div> : null}
        {view === "imports" ? <ImportPanel /> : null}
        {view === "settings" ? <SettingsPanel /> : null}
      </section>
    </main>
  );
}
```

- [ ] **Step 5: Create base styles**

Create `src/styles.css`:

```css
:root {
  color: #1f2933;
  background: #f7f8fa;
  font-family: Inter, "Microsoft YaHei", "PingFang SC", system-ui, sans-serif;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
}

button,
textarea,
input,
select {
  font: inherit;
}

.app-shell {
  display: grid;
  grid-template-columns: 220px 1fr;
  min-height: 100vh;
}

.sidebar {
  border-right: 1px solid #d8dee8;
  background: #ffffff;
  padding: 20px 14px;
}

.brand {
  display: grid;
  gap: 4px;
  margin-bottom: 24px;
}

.brand span {
  color: #667085;
  font-size: 13px;
}

.nav-item {
  width: 100%;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: #344054;
  cursor: pointer;
  margin-bottom: 6px;
  padding: 10px 12px;
  text-align: left;
}

.nav-item.active {
  background: #e8f1ff;
  color: #175cd3;
}

.main-panel {
  min-width: 0;
  padding: 20px;
}

.workspace-path,
.placeholder,
.error-banner {
  border: 1px solid #d8dee8;
  border-radius: 8px;
  margin-bottom: 14px;
  padding: 10px 12px;
}

.workspace-path {
  background: #ffffff;
  color: #667085;
  font-size: 13px;
}

.error-banner {
  background: #fff1f0;
  border-color: #ffccc7;
  color: #b42318;
}
```

- [ ] **Step 6: Commit**

Run:

```bash
git add index.html src/main.tsx src/App.tsx src/styles.css src/lib/workspace.ts
git commit -m "feat: add frontend shell"
```

Expected: commit succeeds.

## Task 4: Local Prompt Types And Formatters

**Files:**
- Create: `src/types/prompt.ts`
- Create: `src/lib/localOptimizer.ts`
- Create: `src/lib/exportFormats.ts`

- [ ] **Step 1: Create shared prompt types**

Create `src/types/prompt.ts`:

```ts
export interface StructuredPrompt {
  subject: string;
  scene: string;
  style: string;
  composition: string;
  camera: string;
  lighting: string;
  color: string;
  details: string;
  negativePrompt: string;
}

export interface OptimizedPrompt {
  zh: string;
  en: string;
  structured: StructuredPrompt;
  matchedTemplateTitles: string[];
}
```

- [ ] **Step 2: Create local optimizer prototype**

Create `src/lib/localOptimizer.ts`:

```ts
import type { OptimizedPrompt, StructuredPrompt } from "../types/prompt";

const qualityTermsZh = ["高细节", "清晰主体", "自然光影", "专业构图"];
const qualityTermsEn = ["high detail", "clear subject", "natural lighting", "professional composition"];

export function optimizePromptLocally(input: string): OptimizedPrompt {
  const trimmed = input.trim();
  const structured: StructuredPrompt = {
    subject: trimmed || "未指定主体",
    scene: inferScene(trimmed),
    style: inferStyle(trimmed),
    composition: "主体明确，画面层次清晰",
    camera: "中景，轻微景深",
    lighting: "柔和自然光",
    color: "色彩协调，对比适中",
    details: qualityTermsZh.join("，"),
    negativePrompt: "低清晰度，画面变形，错误文字，多余肢体，水印",
  };

  return {
    structured,
    zh: renderChinesePrompt(structured),
    en: renderEnglishPrompt(trimmed, structured),
    matchedTemplateTitles: [],
  };
}

function inferScene(input: string): string {
  if (input.includes("雪")) return "雪地或雪山环境";
  if (input.includes("街") || input.includes("城市")) return "城市街景";
  if (input.includes("室内")) return "室内空间";
  return "与主体匹配的自然场景";
}

function inferStyle(input: string): string {
  if (input.includes("电影")) return "电影感";
  if (input.includes("赛博")) return "赛博朋克";
  if (input.includes("水彩")) return "水彩插画";
  if (input.includes("写实")) return "写实摄影";
  return "精致商业视觉";
}

function renderChinesePrompt(prompt: StructuredPrompt): string {
  return [
    prompt.subject,
    prompt.scene,
    prompt.style,
    prompt.composition,
    prompt.camera,
    prompt.lighting,
    prompt.color,
    prompt.details,
  ].join("，");
}

function renderEnglishPrompt(input: string, prompt: StructuredPrompt): string {
  return [
    input || "A clearly defined visual subject",
    `scene: ${prompt.scene}`,
    `style: ${prompt.style}`,
    "balanced composition",
    "medium shot with subtle depth of field",
    "soft natural lighting",
    qualityTermsEn.join(", "),
  ].join(", ");
}
```

- [ ] **Step 3: Create export formatters**

Create `src/lib/exportFormats.ts`:

```ts
import type { OptimizedPrompt } from "../types/prompt";

export type ExportFormat = "gpt-image" | "midjourney" | "stable-diffusion";

export function formatPrompt(prompt: OptimizedPrompt, format: ExportFormat): string {
  if (format === "midjourney") {
    return `${prompt.en} --ar 1:1 --style raw --no watermark, malformed hands, distorted text`;
  }

  if (format === "stable-diffusion") {
    return [
      "Positive Prompt:",
      prompt.en,
      "",
      "Negative Prompt:",
      prompt.structured.negativePrompt,
      "",
      "Suggested Settings:",
      "Size: 1024x1024",
      "Steps: 28",
      "CFG: 6.5",
      "Sampler: DPM++ 2M Karras",
    ].join("\n");
  }

  return `${prompt.en}\n\nAvoid: ${prompt.structured.negativePrompt}`;
}
```

- [ ] **Step 4: Commit**

Run:

```bash
git add src/types/prompt.ts src/lib/localOptimizer.ts src/lib/exportFormats.ts
git commit -m "feat: add local prompt optimization primitives"
```

Expected: commit succeeds.

## Task 5: MVP UI Panels

**Files:**
- Create: `src/components/CreatorWorkspace.tsx`
- Create: `src/components/TemplateLibrary.tsx`
- Create: `src/components/ImportPanel.tsx`
- Create: `src/components/SettingsPanel.tsx`
- Modify: `src/styles.css`

- [ ] **Step 1: Create creator workspace**

Create `src/components/CreatorWorkspace.tsx`:

```tsx
import { useMemo, useState } from "react";
import { formatPrompt, type ExportFormat } from "../lib/exportFormats";
import { optimizePromptLocally } from "../lib/localOptimizer";

export function CreatorWorkspace() {
  const [input, setInput] = useState("一个穿红色斗篷的女孩站在雪山上，电影感");
  const [format, setFormat] = useState<ExportFormat>("gpt-image");
  const result = useMemo(() => optimizePromptLocally(input), [input]);
  const exported = useMemo(() => formatPrompt(result, format), [format, result]);

  return (
    <section className="creator-grid">
      <div className="panel">
        <h2>创作输入</h2>
        <textarea value={input} onChange={(event) => setInput(event.target.value)} />
        <label>
          导出格式
          <select value={format} onChange={(event) => setFormat(event.target.value as ExportFormat)}>
            <option value="gpt-image">GPT Image</option>
            <option value="midjourney">Midjourney</option>
            <option value="stable-diffusion">Stable Diffusion / ComfyUI</option>
          </select>
        </label>
      </div>

      <div className="panel">
        <h2>优化结果</h2>
        <h3>中文提示词</h3>
        <p>{result.zh}</p>
        <h3>英文提示词</h3>
        <p>{result.en}</p>
        <h3>结构化字段</h3>
        <dl className="field-list">
          {Object.entries(result.structured).map(([key, value]) => (
            <div key={key}>
              <dt>{key}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      </div>

      <div className="panel">
        <h2>导出 / 预览</h2>
        <textarea readOnly value={exported} />
        <button>复制提示词</button>
        <button disabled>生成图片：未配置 API</button>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Create template library panel**

Create `src/components/TemplateLibrary.tsx`:

```tsx
export function TemplateLibrary() {
  return (
    <section className="panel">
      <h2>模板库</h2>
      <p>本地提示词模板、收藏、标签和来源信息会显示在这里。</p>
    </section>
  );
}
```

- [ ] **Step 3: Create import panel**

Create `src/components/ImportPanel.tsx`:

```tsx
import { useMemo, useState } from "react";

function classifyGitHubUrl(url: string): string {
  if (url.includes("raw.githubusercontent.com")) return "GitHub raw 文件";
  if (url.includes("/blob/")) return "GitHub blob 文件";
  if (url.includes("github.com")) return "GitHub 仓库";
  return "未知链接";
}

export function ImportPanel() {
  const [url, setUrl] = useState("https://github.com/EvoLinkAI/awesome-gpt-image-2-prompts");
  const type = useMemo(() => classifyGitHubUrl(url), [url]);

  return (
    <section className="panel">
      <h2>参考库导入</h2>
      <input value={url} onChange={(event) => setUrl(event.target.value)} />
      <p>识别结果：{type}</p>
      <button disabled>预览导入：待实现网络适配器</button>
    </section>
  );
}
```

- [ ] **Step 4: Create settings panel**

Create `src/components/SettingsPanel.tsx`:

```tsx
export function SettingsPanel() {
  return (
    <section className="settings-grid">
      <div className="panel">
        <h2>提示词优化 API</h2>
        <select defaultValue="local">
          <option value="local">本地规则模式</option>
          <option value="openai">OpenAI</option>
          <option value="claude">Claude</option>
          <option value="compatible">自定义兼容接口</option>
        </select>
      </div>
      <div className="panel">
        <h2>图片生成 API</h2>
        <select defaultValue="disabled">
          <option value="disabled">未启用</option>
          <option value="gpt-image">GPT Image</option>
          <option value="compatible">自定义图像接口</option>
        </select>
      </div>
    </section>
  );
}
```

- [ ] **Step 5: Extend styles**

Append to `src/styles.css`:

```css
.creator-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(240px, 0.85fr) minmax(360px, 1.3fr) minmax(280px, 1fr);
}

.settings-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(2, minmax(260px, 1fr));
}

.panel {
  background: #ffffff;
  border: 1px solid #d8dee8;
  border-radius: 8px;
  padding: 16px;
}

.panel h2 {
  font-size: 18px;
  margin: 0 0 14px;
}

.panel h3 {
  font-size: 14px;
  margin: 16px 0 8px;
}

textarea,
input,
select {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  display: block;
  margin: 8px 0 14px;
  padding: 10px;
  width: 100%;
}

textarea {
  min-height: 160px;
  resize: vertical;
}

button {
  background: #175cd3;
  border: 0;
  border-radius: 8px;
  color: white;
  cursor: pointer;
  margin-right: 8px;
  padding: 9px 12px;
}

button:disabled {
  background: #98a2b3;
  cursor: not-allowed;
}

.field-list {
  display: grid;
  gap: 8px;
}

.field-list div {
  border-top: 1px solid #eef2f6;
  padding-top: 8px;
}

.field-list dt {
  color: #667085;
  font-size: 12px;
}

.field-list dd {
  margin: 3px 0 0;
}
```

- [ ] **Step 6: Commit**

Run:

```bash
git add src/components/CreatorWorkspace.tsx src/components/TemplateLibrary.tsx src/components/ImportPanel.tsx src/components/SettingsPanel.tsx src/styles.css
git commit -m "feat: add mvp workspace panels"
```

Expected: commit succeeds.

## Task 6: Verification

**Files:**
- Modify only if verification reveals concrete defects.

- [ ] **Step 1: Install dependencies**

Run:

```bash
pnpm install
```

Expected: `node_modules/` is created locally and `pnpm-lock.yaml` is generated.

- [ ] **Step 2: Typecheck frontend**

Run:

```bash
pnpm typecheck
```

Expected: TypeScript exits with code 0.

- [ ] **Step 3: Check Rust code**

Run:

```bash
cd src-tauri
cargo check
```

Expected: Rust compilation exits with code 0.

- [ ] **Step 4: Run desktop app**

Run:

```bash
pnpm dev
```

Expected: PromptWeave opens with the creation workspace, creates `.promptweave/`, and initializes `.promptweave/db.sqlite`.

- [ ] **Step 5: Commit lockfile and verification fixes**

Run:

```bash
git add pnpm-lock.yaml src src-tauri
git commit -m "chore: verify promptweave foundation"
```

Expected: commit succeeds if verification produced lockfiles or fixes.

## Self-Review

- Spec coverage: This plan covers the isolated workspace model, Tauri/React skeleton, local SQLite bootstrap, local optimization prototype, three export formats, two separate API setting areas, and GitHub URL import entry point.
- Deferred by design: Full GitHub network import, Markdown parser, JSON parser, API provider execution, image generation, and history persistence are separate follow-up plans.
- Placeholder scan: The plan contains disabled UI for future behaviors but no implementation placeholder in the task steps themselves.
- Type consistency: `OptimizedPrompt`, `StructuredPrompt`, `ExportFormat`, and `WorkspaceInfo` names are consistent across tasks.
