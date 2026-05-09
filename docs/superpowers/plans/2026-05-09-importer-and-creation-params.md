# Importer And Creation Params Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve GitHub prompt-library import compatibility and add creator-facing generation/export parameters without changing API key storage.

**Architecture:** Extend Rust import parsing with tested Markdown patterns for code blocks, image links, authors, categories, and multiple prompt labels. Add TypeScript creation parameter state that feeds export formatting and image generation request settings.

**Tech Stack:** Rust unit tests, Vitest, React/TypeScript, Tauri commands.

---

## Explicit Non-Goal

- Do not add API key encryption, credential manager integration, or any secure storage change.

## Task 1: Importer Enhancement

- Add tests for fenced prompt code blocks.
- Add tests for Markdown image URL extraction.
- Add tests for author extraction from headings like `Title (by @user)`.
- Add tests for multiple prompt cases inside one category.
- Implement parser changes.

## Task 2: Creation Parameters

- Add typed creation settings for target format, aspect ratio, image size, image quality, image count, Midjourney stylize/chaos, and Stable Diffusion settings.
- Add tests that formatters respect aspect ratio and SD settings.
- Update creator UI controls.
- Pass image size, quality, and count to the Rust image generation command.

## Verification

- `cargo test`
- `cargo check`
- `corepack pnpm test`
- `corepack pnpm typecheck`
- `corepack pnpm vite build`
- `corepack pnpm tauri build --no-bundle`
