# Development commands

Run every task with `cargo make <task>` from the repository root. The root `Makefile.toml` defines all tasks, and each
task runs once for the whole workspace. List them with `cargo make --list-all-steps`.

CI's Windows jobs run the same tasks, so a passing `cargo make verify` means their checks pass too. The desktop tasks
use the Tauri CLI version locked in `bin/xrf-app/ui`, so `cargo-make` is the only tool to install globally.

[ENVIRONMENT.md](./ENVIRONMENT.md) lists every environment variable the binaries and the build read.
`crates/xrf-env-flags` generates it. To add a variable, add it to that crate's table instead of reading it with
`std::env`.

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

Local development and tests use Rust optimization level 0 and line tables only for faster builds: panics and backtraces
name files and lines, but a debugger cannot inspect variables. `serve-backend` and the IDE run configuration
`tauri: debug` build the `debugging` profile, with full debug information, in its own `target/debugging` folder.

Use `cargo make dev-app-optimized` for renderer and viewer work or expensive asset operations: it runs Tauri with the
`dev-optimized` profile (workspace crates at level 1, dependencies at level 3) in its own `target/dev-optimized` folder,
so it never rebuilds what `dev-app` built. The IDE run configuration `tauri: dev-optimized` does the same. Profile with
`cargo make dev-app-profiling`: the `profiling` profile builds release code with line tables, unstripped, in
`target/profiling`, so a profiler measures and names the code that ships.

## Task reference

### Development

| Task                | What it does                                                        |
| ------------------- | ------------------------------------------------------------------- |
| `dev-app`           | Runs the desktop backend and UI in watch mode.                      |
| `dev-app-optimized` | Runs the same watch mode with DLSS and the `dev-optimized` profile. |
| `dev-app-profiling` | Runs the same watch mode with DLSS and the `profiling` profile.     |
| `serve-backend`     | Runs the desktop backend without the UI dev server, for a debugger. |
| `serve-ui`          | Runs the Vite UI dev server without the backend.                    |

### Build

| Task                | What it does                                                                                                                   |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `build-cli`         | Builds the CLI in debug mode.                                                                                                  |
| `build-cli-release` | Builds the CLI in release mode.                                                                                                |
| `build-app`         | Builds the desktop application in debug mode, without DLSS.                                                                    |
| `build-app-release` | Builds the desktop application in release mode, with DLSS, from cleared resources. Sets `XRF_BUILD_KIND=optimized` unless set. |
| `package-app`       | Builds the release application and packs it into `target/package/<PACKAGE_NAME>.zip`.                                          |
| `fetch-sdks`        | Fetches the pinned DLSS SDK, Vulkan headers and libclang into `.sdks/`.                                                        |

`package-app` names the archive `xrf-app-windows-x64` unless `PACKAGE_NAME` is set. `fetch-sdks` skips anything
already at its pinned version, so repeated runs fetch nothing.

### Verification

| Task           | What it does                                                                                        |
| -------------- | --------------------------------------------------------------------------------------------------- |
| `format`       | Formats the Rust codebase.                                                                          |
| `check-format` | Checks Rust formatting without writing files.                                                       |
| `lint`         | Runs Clippy on all workspace targets with DLSS. Fetches the SDKs first.                             |
| `test`         | Runs workspace library and binary tests with DLSS and checks the generated frontend bindings.       |
| `test-derive`  | Runs every derive crate's UI diagnostic snapshots (each crate's `tests/ui.rs`).                     |
| `test-doc`     | Runs the documentation examples of every workspace crate, READMEs included.                         |
| `install-ui`   | Installs the locked TypeScript dependencies.                                                        |
| `test-ui`      | Runs the tests of every TypeScript workspace member.                                                |
| `verify-ui`    | Installs locked TypeScript dependencies, then typechecks, tests, and lints every workspace member.  |
| `verify`       | Runs the CI-gated suite: format check, lint, workspace tests, derive diagnostics, doc examples, UI. |

`lint` and `test` enable the Cargo features listed in `CHECK_FEATURES` in `Makefile.toml`. Change the list there;
CI follows because it runs the same tasks.

### Documentation and generated code

| Task                           | What it does                                                                                  |
| ------------------------------ | --------------------------------------------------------------------------------------------- |
| `doc`                          | Generates Rust documentation for workspace members, without dependencies.                     |
| `doc-cli`                      | Generates the CLI command reference from the Clap definitions into `target/doc-commands`.     |
| `generate-typescript-bindings` | Emits the generated TypeScript mirrors and Tauri command bindings.                            |
| `format-typescript-bindings`   | Formats and lints the generated TypeScript bindings.                                          |
| `generate-typescript`          | Regenerates and formats the TypeScript bindings. Use it after changing IPC types or commands. |

### Maintenance

| Task    | What it does                       |
| ------- | ---------------------------------- |
| `clean` | Removes workspace build artifacts. |

## Prerequisites

See the [build requirements](./README.md#build-from-source). The desktop app needs the Rust toolchain, Node.js and
pnpm; the Tauri CLI comes locked with `bin/xrf-app/ui`. The CLI-only tasks need Rust and Cargo Make.

The renderer's water and rain maps (`crates/xrf-renderer/assets/water`, `assets/rain`) are stored in Git LFS. Install
it before cloning (`git lfs install`), or run `git lfs pull` after: without it the tree holds pointer files in their
place and `package-app` refuses to pack.

On Windows the application ships DXC, which D3D12 compiles shader model 6 with: `cargo make fetch-sdks` places the
pinned release in `.sdks/dxc`, and `tauri.windows.conf.json` copies it into the renderer's bundle. Every task that
builds the application fetches it first; a plain `cargo build -p xrf-app` wants it fetched once.

### DLSS

The desktop app upscales with NVIDIA DLSS when built with the `dlss` feature (`cargo build -p xrf-app --features dlss`),
on an RTX GPU, with either graphics API. `dev-app` and `build-app` build without it and need none of the setup below.
`build-app-release`, `dev-app-optimized`, `dev-app-profiling`, `lint` and `test` build with it.

- `cargo make fetch-sdks` checks the DLSS SDK and Vulkan's C headers out into `.sdks/`, each pinned to a commit, only
  the files the build reads, and on Windows places LLVM's libclang there for bindgen. Whatever is already there at the
  pinned commit or hash is left alone, so it fetches nothing on a second run. Every task that builds with DLSS runs it
  first. `.cargo/config.toml` points `DLSS_SDK`, `VULKAN_SDK` and `LIBCLANG_PATH` there; any set in the environment wins.
- Elsewhere than Windows, libclang comes from the system (libclang-dev or LLVM), or `LIBCLANG_PATH`.
- A build with the feature copies `nvngx_dlss.dll`, `nvngx_dlssd.dll` and NVIDIA's `LICENSE.txt` into its profile's
  `resources/renderer/dlss/` (`bin/xrf-app/app/tauri.dlss.conf.json`). NVIDIA's licence lets them ship only inside the
  application, under terms as protective as its own, so they are never committed.
- wgpu and naga come from the `xray-forge/xrf-wgpu` fork (`[patch.crates-io]` in `Cargo.toml`): a wgpu release with
  its commits on top: the D3D12 command list a HAL encoder records into exposed, which NGX records DLSS into, with the
  conditions it is safe under; and bottom-level acceleration structures refit in place where they allow and ask for it,
  only from a build laid out as their last one (`blas_refit_shrunk` in its GPU tests). Moving to a newer wgpu means
  rebasing those commits onto its tag in the fork and pinning the new one.

### Tracy

A capture build for [Tracy](https://github.com/wolfpld/tracy) is the `profiling` profile (release code with line
tables) with the app's `tracy` feature, the renderer's zones, GPU zones and frame marks, built with the `tracy`
profile into a target folder of its own, so switching never rebuilds the others:
`cargo build -p xrf-app --profile tracy --features tracy,dlss`. Tracy's profiler connects to the running app.
