# Template Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let creators manage imported prompt references by favoriting, editing metadata/content, and archiving templates from the local library.

**Architecture:** Extend the existing SQLite `prompt_templates` table with compatibility columns for favorite/archive state, expose small Tauri commands around focused DB helpers, and keep the React template library as the single management surface. Archived records are hidden from list/search so normal retrieval remains clean.

**Tech Stack:** Tauri 2, Rust, rusqlite/SQLite FTS5, React 19, TypeScript, Vitest.

---

### Task 1: Backend Template State

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] Add failing Rust tests proving favorite toggles, edits update searchable content, and archived templates are hidden from list/search.
- [ ] Run `cargo test template` and confirm the new tests fail because the functions/fields do not exist.
- [ ] Add `is_favorite` and `is_archived` compatibility columns in `bootstrap`.
- [ ] Extend `PromptTemplateRecord` and row mapping.
- [ ] Implement `TemplateUpdateDraft`, `update_prompt_template`, `toggle_prompt_template_favorite`, and `archive_prompt_template`.
- [ ] Run `cargo test template` and confirm the tests pass.

### Task 2: Tauri Commands

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] Add command wrappers for update, favorite toggle, and archive.
- [ ] Register commands in `tauri::generate_handler!`.
- [ ] Run `cargo check`.

### Task 3: Frontend Library UI

**Files:**
- Modify: `src/components/TemplateLibrary.tsx`
- Modify: `src/styles.css`

- [ ] Add `isFavorite` to the frontend record type.
- [ ] Add a favorites-only checkbox and favorite toggle action.
- [ ] Add row edit mode for title/category/prompt/negative/aspect/tags with save/cancel.
- [ ] Add archive action that hides the template after confirmation.
- [ ] Keep controls compact and consistent with the existing app style.
- [ ] Run `corepack pnpm typecheck` and `corepack pnpm vite build`.

### Task 4: Final Verification and Integration

**Files:**
- No planned source edits.

- [ ] Run `cargo test`.
- [ ] Run `cargo check`.
- [ ] Run `corepack pnpm test`.
- [ ] Run `corepack pnpm typecheck`.
- [ ] Run `corepack pnpm vite build`.
- [ ] Run `corepack pnpm tauri build --no-bundle`.
- [ ] Commit the feature branch.
- [ ] Merge `codex/template-management` back into `main`.
