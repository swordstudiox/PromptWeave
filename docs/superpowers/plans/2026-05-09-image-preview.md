# Image Preview Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the creator preview column useful by enabling copy-to-clipboard and API-backed image generation that saves generated images to the isolated workspace.

**Architecture:** Add a Rust generation module for image API request construction, config validation, response parsing, and image saving. The creator UI calls one Tauri command and renders the saved local image path.

**Tech Stack:** Rust unit tests, Tauri commands, ureq, base64, React/TypeScript.

---

## Tasks

- [ ] Write failing Rust tests for disabled image config and default request construction.
- [ ] Implement image generation helpers and Tauri command.
- [ ] Update creator UI copy and generate buttons.
- [ ] Run `cargo test`, `cargo check`, `corepack pnpm test`, `corepack pnpm typecheck`, and `corepack pnpm vite build`.
- [ ] Commit image preview support.
