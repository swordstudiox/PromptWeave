# Library Source Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist GitHub prompt library sources and let users manually re-sync saved sources from the import page.

**Architecture:** Add a lightweight `prompt_library_sources` table that tracks import URL, type, display name, last sync counts, last error, and timestamps. Existing import parsing and template insertion stay unchanged; import success records the source, and manual sync reuses the same import pipeline by source id.

**Tech Stack:** Tauri 2, Rust, rusqlite/SQLite, React 19, TypeScript.

---

### Task 1: Source Metadata Persistence

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] Add failing tests for upserting a source, recording successful sync counts, recording sync errors, and listing most recent sources first.
- [ ] Run `cargo test library_source` and confirm the tests fail because source structs/functions do not exist.
- [ ] Add `prompt_library_sources` table in `bootstrap`.
- [ ] Add source record/draft structs and helper functions for upsert, sync success, sync error, and list.
- [ ] Run `cargo test library_source` and confirm the tests pass.

### Task 2: Import Pipeline Integration

**Files:**
- Modify: `src-tauri/src/imports.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] Extend `ImportResult` with `sourceId`.
- [ ] Record source metadata after successful import.
- [ ] Add `list_prompt_library_sources` and `sync_prompt_library_source` commands.
- [ ] Run `cargo check`.

### Task 3: Import Page Source List

**Files:**
- Modify: `src/components/ImportPanel.tsx`
- Modify: `src/styles.css`

- [ ] Load saved sources on page mount.
- [ ] Show source name, URL, type, last sync summary, and last error.
- [ ] Add per-source sync button that calls `sync_prompt_library_source`.
- [ ] Refresh source list after normal import and manual sync.
- [ ] Run `corepack pnpm typecheck` and `corepack pnpm vite build`.

### Task 4: Final Verification and Merge

**Files:**
- No planned source edits.

- [ ] Run `cargo test`.
- [ ] Run `cargo check`.
- [ ] Run `corepack pnpm test`.
- [ ] Run `corepack pnpm typecheck`.
- [ ] Run `corepack pnpm vite build`.
- [ ] Run `corepack pnpm tauri build --no-bundle`.
- [ ] Commit the feature branch.
- [ ] Merge `codex/library-sync` back into `main`.
