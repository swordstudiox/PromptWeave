# Template Assisted Optimizer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the creator workspace use imported local templates as references when optimizing prompts.

**Architecture:** Keep the optimizer pure in TypeScript by accepting optional template references. The creator page queries the Rust template search command and passes matches into the optimizer, then renders matched template names.

**Tech Stack:** Vitest, React/TypeScript, Tauri commands.

---

## Tasks

- [ ] Write failing Vitest tests for template-assisted optimization.
- [ ] Update optimizer types and implementation to use template references.
- [ ] Update creator workspace to search templates and show matches.
- [ ] Run `corepack pnpm test`, `corepack pnpm typecheck`, `corepack pnpm vite build`, `cargo test`, and `cargo check`.
- [ ] Commit the optimizer feature.
