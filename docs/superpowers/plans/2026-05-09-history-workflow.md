# History Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist and display creation history so users can revisit inputs, optimized prompts, export format, matched templates, parameters, and generated image paths.

**Architecture:** Store history in SQLite through Rust commands. The creator page records history after copy, API optimization, or image generation. The history page lists records and can load a record back into the creator page.

**Tech Stack:** Rust unit tests, rusqlite, Tauri commands, React/TypeScript.

---

## Tasks

- [ ] Write failing Rust tests for saving and listing generation history.
- [ ] Add a complete history schema and DB helpers.
- [ ] Expose `save_generation_history` and `list_generation_history`.
- [ ] Replace the placeholder history page with a real list.
- [ ] Let history records load back into the creator input.
- [ ] Run `cargo test`, `cargo check`, `corepack pnpm test`, `corepack pnpm typecheck`, `corepack pnpm vite build`, and `corepack pnpm tauri build --no-bundle`.
- [ ] Commit the history workflow.
