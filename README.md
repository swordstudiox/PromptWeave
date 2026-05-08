# PromptWeave

PromptWeave is a lightweight offline-first image prompt workspace for creators.

## Isolation Model

PromptWeave stores runtime data inside a workspace-local `.promptweave/` directory:

- `db.sqlite` for local prompt templates and history
- `cache/` for imported source files
- `imports/` for GitHub import metadata
- `exports/` for copied/exported prompt files
- `history/` for generated prompt and image records

This keeps each workspace self-contained, similar to a Python virtual environment.

## Development

Use the project-local package manager lockfile and Tauri bundle. Do not require users to install Python, a separate database server, or global app services.

```bash
corepack pnpm install
corepack pnpm dev
```

Developers who build the Tauri desktop shell need Rust installed. The repository pins the Rust toolchain through `rust-toolchain.toml`; end users of packaged builds do not need Rust or Node.

Useful verification commands:

```bash
corepack pnpm typecheck
corepack pnpm vite build
cd src-tauri
cargo check
```

If `cargo` is not available, install Rust with rustup before running Tauri commands.
