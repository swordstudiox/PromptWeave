# Prompt Optimization API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users optionally send the local structured prompt result to a configured prompt optimization API and use the returned prompt in export/generation.

**Architecture:** Add a Rust prompt API module that validates separate prompt optimization settings and supports OpenAI-compatible chat completions plus Claude Messages. The creator UI keeps local optimization as the baseline and allows API-enhanced text to override the export text.

**Tech Stack:** Rust unit tests, ureq, serde_json, Tauri commands, React/TypeScript.

---

## Tasks

- [ ] Write failing Rust tests for disabled prompt API and OpenAI-compatible request construction.
- [ ] Implement prompt optimization request builder and Tauri command.
- [ ] Update creator UI with API optimization button and override output.
- [ ] Run `cargo test`, `cargo check`, `corepack pnpm test`, `corepack pnpm typecheck`, and `corepack pnpm vite build`.
- [ ] Commit prompt optimization API support.
