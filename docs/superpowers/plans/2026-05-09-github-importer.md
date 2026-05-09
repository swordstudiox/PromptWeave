# GitHub Importer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users paste a GitHub repository, blob, raw Markdown, or raw JSON link, preview extracted prompt templates, and import them into the local SQLite library.

**Architecture:** Keep GitHub import logic in Rust so local storage, URL normalization, network fetching, parsing, and SQLite insertion share one boundary. The React import page calls Tauri commands for preview/import and only renders results.

**Tech Stack:** Tauri commands, Rust unit tests, reqwest blocking client, serde_json, rusqlite, React/TypeScript.

---

## File Structure

- Modify: `src-tauri/Cargo.toml`
  Add HTTP and hashing dependencies.
- Modify: `src-tauri/src/imports.rs`
  Add GitHub URL parsing, candidate source resolution, Markdown/JSON prompt parsing, preview/import types, tests.
- Modify: `src-tauri/src/db.rs`
  Add insert helpers for parsed prompt templates.
- Modify: `src-tauri/src/main.rs`
  Expose `preview_import_url` and `import_prompt_library` commands.
- Modify: `src/components/ImportPanel.tsx`
  Add preview table, import button, loading state, error state.

## Tasks

- [ ] Write failing Rust tests for GitHub URL classification and source resolution.
- [ ] Implement URL classification/resolution until tests pass.
- [ ] Write failing Rust tests for Markdown prompt extraction.
- [ ] Implement Markdown prompt extraction until tests pass.
- [ ] Write failing Rust tests for JSON prompt extraction.
- [ ] Implement JSON prompt extraction until tests pass.
- [ ] Add SQLite insert helper and import command.
- [ ] Update React import panel.
- [ ] Run `cargo test`, `cargo check`, `corepack pnpm typecheck`, and `corepack pnpm vite build`.
- [ ] Commit the importer feature.

## Self-Review

- Covers GitHub repo/blob/raw links.
- Covers Markdown and JSON.
- Keeps network optional at runtime; local prompt generation still works without import.
- Does not add account, cloud sync, or background scheduler.
