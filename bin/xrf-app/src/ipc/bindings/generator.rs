//! Driving `xrf-ipc-typescript` over this application's command surface.
//!
//! Generation itself belongs to that crate, including the order its steps run in; what is decided here is only which
//! surfaces go in and where the output lands.
//!
//! Both entry points are `#[test]` because `cargo test` is the wrapper that has the Specta builders linked in;
//! `export_typescript_bindings` is the generator and is `#[ignore]`d so only `cargo make generate-typescript` runs
//! it, while `verify_typescript_bindings` is an ordinary test of the committed mirrors.

use std::path::PathBuf;

use xrf_ipc_typescript::{IpcBindingsGenerator, SurfaceDrift};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::ipc::bindings::command_surfaces::command_surfaces;

/// Path of the mirrors the frontend compiles against.
fn bindings_output_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../xrf-ui/src/core/ipc")
}

/// Rewrite the committed mirrors in place.
#[test]
#[ignore]
fn export_typescript_bindings() {
  IpcBindingsGenerator::generate(&bindings_output_path(), &command_surfaces::<tauri::Wry>());
}

/// Fail only where the committed mirrors no longer describe what the Rust sources produce.
#[test]
fn verify_typescript_bindings() {
  let drift: SurfaceDrift = IpcBindingsGenerator::verify(
    &bindings_output_path(),
    &build_absolute_generated_test_resource_path("bindings"),
    &command_surfaces::<tauri::Wry>(),
  );

  assert!(!drift.is_breaking(), "{}", drift.describe());
}
