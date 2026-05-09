# Template Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show imported prompt templates in the Template Library page and support local search over the SQLite-backed library.

**Architecture:** Add Rust database query commands for listing/searching templates, keep SQLite FTS maintenance in the database bootstrap, and update React state to call Tauri commands. Fix workspace JSON naming to match the Tauri camelCase payload.

**Tech Stack:** Tauri commands, Rust unit tests, rusqlite, React/TypeScript.

---

## Tasks

- [ ] Write failing Rust tests for insert + list + FTS search.
- [ ] Add SQLite FTS triggers and query helpers.
- [ ] Expose `list_prompt_templates` and `search_prompt_templates`.
- [ ] Update `TemplateLibrary.tsx` to load and search local templates.
- [ ] Fix `WorkspaceInfo` TypeScript names from snake_case to camelCase.
- [ ] Run `cargo test`, `cargo check`, `corepack pnpm typecheck`, and `corepack pnpm vite build`.
- [ ] Commit the template library feature.
