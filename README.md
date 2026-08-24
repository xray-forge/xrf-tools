# 🎮 [Stalker XRF Tools](README.md)

[![book](https://img.shields.io/badge/docs-book-blue.svg?style=flat)](https://xray-forge.github.io/xrf-book)
[![language-rust](https://img.shields.io/badge/language-rust-orange.svg?style=flat)](https://github.com/xray-forge/xrf-tools/search?l=rust)
[![license](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](https://github.com/Neloreck/dreamstate/blob/master/LICENSE)
<br/>
[![build and test](https://github.com/xray-forge/xrf-tools/actions/workflows/build_and_test.yml/badge.svg?branch=main)](https://github.com/xray-forge/xrf-tools/actions/workflows/build_and_test.yml)

Set of [utility tools](https://xray-forge.github.io/xrf-book/tools/tools.html) to assist with xray engine mods
development and debugging. <br/>
Includes UI application (windows) for usability and manual usage and CLI (windows, linux) variant for scripts / CI.

## Application

Documented in [xrf book](https://xray-forge.github.io/xrf-book/tools/app/app.html).

- [Archive editor](https://xray-forge.github.io/xrf-book/tools/app/archive_editor.md)
- [Dialog editor](https://xray-forge.github.io/xrf-book/tools/app/dialog_editor.md)
- [Config editor](https://xray-forge.github.io/xrf-book/tools/app/config_editor.md)
- [Exports viewer](https://xray-forge.github.io/xrf-book/tools/app/exports_viewer.md)
- [Icon editor](https://xray-forge.github.io/xrf-book/tools/app/icon_editor.md)
- [Spawn editor](https://xray-forge.github.io/xrf-book/tools/app/spawn_editor.md)

<img width="600px" src="https://xray-forge.github.io/xrf-book/tools/app/images/main_window.png">

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
