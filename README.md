<img width="600px" src="https://xray-forge.github.io/xrf-book/images/xrf-tools-banner%400.5x.png">

# XRF Tools

[![book](https://img.shields.io/badge/docs-book-blue.svg?style=flat)](https://xray-forge.github.io/xrf-book)
[![language-rust](https://img.shields.io/badge/language-rust-orange.svg?style=flat)](https://github.com/xray-forge/xrf-tools/search?l=rust)
[![license](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](https://github.com/Neloreck/dreamstate/blob/master/LICENSE)
<br/>
[![build and test](https://github.com/xray-forge/xrf-tools/actions/workflows/build_and_test.yml/badge.svg?branch=main)](https://github.com/xray-forge/xrf-tools/actions/workflows/build_and_test.yml)

XRF Tools is an open-source Windows application and cross-platform CLI for browsing, validating, editing, packing and
converting S.T.A.L.K.E.R. X-Ray Engine assets.

## Application

Documented in [xrf book](https://xray-forge.github.io/xrf-book/tools/app/app.html).

<img width="600px" src="https://xray-forge.github.io/xrf-book/images/xrf-app-main-window.png">

## CLI

Documented in [xrf book](https://xray-forge.github.io/xrf-book/tools/cli/cli.html).\
Command reference is generated from clap definitions with `cargo make docs-cli` into `target/commands-docs`.

- [Archive commands](https://xray-forge.github.io/xrf-book/tools/cli/archive.html)
- [Gamedata commands](https://xray-forge.github.io/xrf-book/tools/cli/gamedata.html)
- [Icons commands](https://xray-forge.github.io/xrf-book/tools/cli/icons.html)
- [LTX commands](https://xray-forge.github.io/xrf-book/tools/cli/ltx.html)
- [OGF commands](https://xray-forge.github.io/xrf-book/tools/cli/ogf.html)
- [OMF commands](https://xray-forge.github.io/xrf-book/tools/cli/omf.html)
- [Particles commands](https://xray-forge.github.io/xrf-book/tools/cli/particles.html)
- [Spawn commands](https://xray-forge.github.io/xrf-book/tools/cli/spawn.html)
- [Translations commands](https://xray-forge.github.io/xrf-book/tools/cli/translations.html)

## Building

### Requirements

- node-js
- rust
- cargo-make (`cargo install --force cargo-make`)
- tauri-cli (`cargo install --force tauri-cli@2.5.0`),
  [tauri installation](https://tauri.app/v1/guides/getting-started/prerequisites)

### Release

APP: `cargo make build-app-release`\
CLI: `cargo make build-cli-release`

### Dev

APP: `cargo tauri dev`\
APP-BACKEND: `cargo make serve-backend`\
APP-UI: `cargo make serve-ui`\
CLI: `cargo make build-cli`
