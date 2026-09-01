<img src="https://xray-forge.github.io/xrf-book/images/xrf-tools-banner%400.5x.png" alt="XRF Tools">

# XRF Tools

[![nightly build](https://github.com/xray-forge/xrf-tools/actions/workflows/build_and_test.yml/badge.svg?branch=main)](https://github.com/xray-forge/xrf-tools/actions/workflows/build_and_test.yml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](./LICENSE)
[![language-rust](https://img.shields.io/badge/language-Rust-orange.svg?style=flat)](https://www.rust-lang.org/)

XRF Tools is an open-source desktop application and command-line toolkit for reading, validating, editing, packing, and
converting assets for S.T.A.L.K.E.R.: Call of Pripyat–style X-Ray games.

## Get XRF Tools

[Download the nightly builds](https://github.com/xray-forge/xrf-tools/releases/tag/nightly).

- **Windows:** desktop application and CLI.
- **Linux:** CLI.

## What it includes

### Desktop application

The Windows application provides focused workspaces for:

- browsing, extracting, and packing X-Ray archive volumes;
- browsing LTX configs, dialog trees, quests, info portions, character profiles, and script exports, and formatting or
  validating LTX files;
- previewing OGF visuals and their motions in 3D;
- inspecting, unpacking and packing spawn files and icon sprite sheets;
- parsing, editing, building, and checking translations;
- validating assembled `gamedata` trees.

<img width="600" src="https://xray-forge.github.io/xrf-book/images/xrf-app-main-window.png" alt="XRF Tools desktop application">

### Command-line interface

`xrf-cli` is for repeatable local work, scripts, and CI. Its command groups cover X-Ray archives, assembled game data,
LTX configurations, dialogs and script externs, DDS and THM textures, OGF models, OMF motions, particle libraries, spawn
files, icon sprites, and translations. Commands can emit JSON reports for automation.

The [CLI reference](https://xray-forge.github.io/xrf-book/tools/cli/cli.html) documents every command and option.

## Compatibility

XRF Tools targets vanilla Call of Pripyat assets first. Support is expanding to other X-Ray 1.6–based forks, whose
format and engine differences can vary by project. Validate write operations against a copy of the assets you intend to
ship.

## Build from source

The repository is a Rust workspace with a Tauri desktop app and React UI.

### Install prerequisites

The CLI needs Rust 1.97.1 and [Cargo Make](https://github.com/sagiegurari/cargo-make). The desktop app also needs
Node.js 24 or later, pnpm 11.9.0, and the [Tauri CLI](https://v2.tauri.app/start/prerequisites/).

After installing Rust, Node.js, and pnpm, install the project tools and locked UI dependencies:

```powershell
rustup toolchain install 1.97.1
cargo install cargo-make --locked
cargo install tauri-cli --locked
pnpm --dir bin/xrf-ui install --frozen-lockfile
```

### Build and run

```powershell
cargo make build-cli-release
cargo make build-app-release
```

For desktop development, run the backend and UI together:

```powershell
cargo make dev-app
```

See [Development commands](./DEVELOPMENT.md) for the complete task reference.

Run the complete local check suite before contributing a change:

```powershell
cargo make verify
```

CLI e2e tests live in the [xrf-tools-e2e repository](https://github.com/xray-forge/xrf-tools-e2e).

## Documentation and support

- [XRF Book](https://xray-forge.github.io/xrf-book/tools/tools.html) — guides for the application and CLI.
- [CLI reference](https://xray-forge.github.io/xrf-book/tools/cli/cli.html) — generated command and option reference.
- [Issue tracker](https://github.com/xray-forge/xrf-tools/issues) — report a reproducible problem or propose an
  improvement.
- [MIT License](./LICENSE)
