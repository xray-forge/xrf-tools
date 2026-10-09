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

Local development and tests use Rust optimization level 0 for faster builds. Use `cargo make dev-app-optimized` for
renderer and viewer work or expensive asset operations: it runs Tauri with the `dev-optimized` profile (workspace crates
at level 1, dependencies at level 3) in its own `target/dev-optimized` folder, so it never rebuilds what `dev-app`
built. The IDE run configuration `tauri: dev-optimized` does the same. Profile with `cargo make dev-app-profiling`: the
`profiling` profile builds release code with line tables, unstripped, in `target/profiling`, so a profiler measures and
names the code that ships.

## Task reference

### Development

| Task                | What it does                                               |
| ------------------- | ---------------------------------------------------------- |
| `dev-app`           | Runs the desktop backend and UI in watch mode.             |
| `dev-app-optimized` | Runs the same watch mode with the `dev-optimized` profile. |
| `dev-app-profiling` | Runs the same watch mode with the `profiling` profile.     |
| `serve-backend`     | Runs the desktop backend without the UI dev server.        |
| `serve-ui`          | Runs the Vite UI dev server without the backend.           |

### Build

| Task                | What it does                                                                              |
| ------------------- | ----------------------------------------------------------------------------------------- |
| `build-cli`         | Builds the CLI in debug mode.                                                             |
| `build-cli-release` | Builds the CLI in release mode.                                                           |
| `build-app`         | Builds the desktop application in debug mode.                                             |
| `build-app-release` | Builds the desktop application in release mode and installs locked UI dependencies first. |
| `fetch-sdks`        | Fetches the SDKs the optional native features build against into `.sdks/`.                |

### Verification

| Task                | What it does                                                                                          |
| ------------------- | ----------------------------------------------------------------------------------------------------- |
| `format`            | Formats the Rust codebase.                                                                            |
| `format-check`      | Checks Rust formatting without writing files.                                                         |
| `lint`              | Runs Clippy across the workspace and all targets.                                                     |
| `test`              | Runs Cargo Make's workspace test flow.                                                                |
| `test-workspace`    | Runs workspace library and binary tests and checks generated frontend bindings.                       |
| `test-error-derive` | Runs the `xrf-error-derive` UI diagnostic snapshots.                                                  |
| `verify-ui`         | Installs locked TypeScript dependencies, then typechecks, tests, and lints every workspace member.    |
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

### DLSS

The desktop app upscales with NVIDIA DLSS when built with the `dlss` feature (`cargo build -p xrf-app --features dlss`),
on an RTX GPU, with either graphics API. Development builds leave it off and need nothing below; `cargo make
build-app-release` builds with it.

- `cargo make fetch-sdks` checks the DLSS SDK and Vulkan's C headers out into `.sdks/`, each pinned to a commit, only
  the files the build reads, and on Windows places LLVM's libclang there for bindgen. Whatever is already there at the
  pinned commit or hash is left alone, so it fetches nothing on a second run; `build-app-release` runs it first.
  `.cargo/config.toml` points `DLSS_SDK`, `VULKAN_SDK` and `LIBCLANG_PATH` there; any set in the environment wins.
- Elsewhere than Windows, libclang comes from the system (libclang-dev or LLVM), or `LIBCLANG_PATH`.
- A build with the feature copies `nvngx_dlss.dll` into its profile's `resources/renderer/dlss/`
  (`bin/xrf-app/tauri.dlss.conf.json`), which the release installer carries too. NVIDIA's licence lets it ship only
  inside the application, so it is never committed.
- wgpu and naga come from the `xray-forge/xrf-wgpu` fork (`[patch.crates-io]` in `Cargo.toml`): a wgpu release with
  one commit on top that exposes the D3D12 command list a HAL encoder records into, which NGX records DLSS into. Moving
  to a newer wgpu means rebasing that commit onto its tag in the fork and pinning the new commit.
