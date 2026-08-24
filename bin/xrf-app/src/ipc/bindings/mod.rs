//! Generation of the frontend's TypeScript mirrors of the Rust types and Tauri commands.
//!
//! Output lands in `bin/xrf-ui/src/core/bindings/`, split into `types/` and `commands/`. A type is written
//! once, into the module of the crate declaring it, which is read off `module_path!()` rather than from any
//! hand-written list. Run it with `cargo make generate-typescript`.

mod command_module;
mod constants;
mod exporter;
mod normalization;
mod output;
mod ownership;
mod references;
mod surface;
mod types_module;

#[cfg(test)]
mod tests;
