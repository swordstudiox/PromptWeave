# API Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist separate prompt optimization API and image generation API settings inside the workspace-local `.promptweave/config.json`.

**Architecture:** Add a Rust config module that reads/writes local JSON under the isolated workspace directory. The React settings page loads and saves the two provider configs independently through Tauri commands.

**Tech Stack:** Rust serde_json, Tauri commands, React/TypeScript.

---

## Tasks

- [ ] Write failing Rust tests for default config and save/load round trip.
- [ ] Implement workspace config read/write.
- [ ] Expose `get_app_config` and `save_app_config` commands.
- [ ] Update settings UI with provider, base URL, model, API key, and enabled controls.
- [ ] Run `cargo test`, `cargo check`, `corepack pnpm typecheck`, and `corepack pnpm vite build`.
- [ ] Commit API settings persistence.
