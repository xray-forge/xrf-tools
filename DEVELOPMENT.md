# Development commands

Run `cargo make <task>` from the repository root. The root tasks are the supported contributor entry points; they
coordinate the CLI, desktop backend, and UI where necessary.

## Common workflows

| Goal                                        | Command                        |
| ------------------------------------------- | ------------------------------ |
| Develop the desktop app                     | `cargo make dev-app`           |
| Develop the desktop app with optimized Rust | `cargo make dev-app-optimized` |
| Build the CLI for local use                 | `cargo make build-cli`         |
| Build the desktop app for local use         | `cargo make build-app`         |
| Run the complete pre-push check suite       | `cargo make verify`            |

`dev-app` starts the Tauri backend and the Vite UI in watch mode. Use `serve-backend` or `serve-ui` when working on one
layer independently or need to attach debugger.

Local development and tests use Rust optimization level 0 for faster builds. Use `cargo make dev-app-optimized` to
run Tauri with optimization level 1 when working with expensive asset operations. This command trades longer builds
for faster backend execution and sets the optimization override only for its process and children.

## Task reference

### Development

| Task                | What it does                                             |
| ------------------- | -------------------------------------------------------- |
| `dev-app`           | Runs the desktop backend and UI in watch mode.           |
| `dev-app-optimized` | Runs the same watch mode with Rust optimization level 1. |
| `serve-backend`     | Runs the desktop backend without the UI dev server.      |
| `serve-ui`          | Runs the Vite UI dev server without the backend.         |

### Build

| Task                | What it does                                                                              |
| ------------------- | ----------------------------------------------------------------------------------------- |
| `build-cli`         | Builds the CLI in debug mode.                                                             |
| `build-cli-release` | Builds the CLI in release mode.                                                           |
| `build-app`         | Builds the desktop application in debug mode.                                             |
| `build-app-release` | Builds the desktop application in release mode and installs locked UI dependencies first. |

### Verification

| Task                | What it does                                                                                          |
| ------------------- | ----------------------------------------------------------------------------------------------------- |
| `format`            | Formats the Rust codebase.                                                                            |
| `format-check`      | Checks Rust formatting without writing files.                                                         |
| `lint`              | Runs Clippy across the workspace and all targets.                                                     |
| `test`              | Runs Cargo Make's workspace test flow.                                                                |
| `test-workspace`    | Runs workspace library and binary tests and checks generated frontend bindings.                       |
| `test-error-derive` | Runs the `xrf-error-derive` UI diagnostic snapshots.                                                  |
| `verify-ui`         | Installs locked UI dependencies, then typechecks, tests, and lints the UI.                            |
| `verify`            | Runs the full CI-gated suite: format check, lint, workspace tests, derive diagnostics, and UI checks. |

### Documentation and generated code

| Task                           | What it does                                                                                  |
| ------------------------------ | --------------------------------------------------------------------------------------------- |
| `doc`                          | Generates Rust documentation for workspace members.                                           |
| `docs-cli`                     | Generates the CLI command reference from the Clap definitions into `target/doc-commands`.     |
| `generate-typescript-bindings` | Emits the generated TypeScript mirrors and Tauri command bindings.                            |
| `format-typescript-bindings`   | Formats and lints the generated TypeScript bindings.                                          |
| `generate-typescript`          | Regenerates and formats the TypeScript bindings. Use it after changing IPC types or commands. |

### Maintenance

| Task    | What it does                       |
| ------- | ---------------------------------- |
| `clean` | Removes workspace build artifacts. |

## Prerequisites

See the [build requirements](./README.md#build-from-source). The desktop app needs the Rust toolchain, Node.js, pnpm,
and the Tauri CLI. The CLI-only tasks need Rust and Cargo Make.
