//! Driving `xrf-ipc-typescript` over this application's command surface.
//!
//! Generation itself belongs to that crate, including the order its steps run in; what is decided here is only which
//! surfaces go in and where the output lands.
//!
//! Both entry points are `#[test]` because `cargo test` is the wrapper that has the Specta builders linked in;
//! `export_typescript_bindings` is the generator and is `#[ignore]`d so only `cargo make generate-typescript` runs
//! it, while `verify_typescript_bindings` is an ordinary test of the committed mirrors.

use std::path::PathBuf;

use xrf_ipc_typescript::{IpcBindingsGenerator, MirrorImports, SurfaceDrift, TypeModules};
use xrf_test_utils::GeneratedResources;

use crate::ipc::bindings::command_surfaces::command_surfaces;

/// How the mirror's modules address one another and the runtime beside them: through the package's own `#/*` mapping.
const MIRROR_IMPORTS: MirrorImports = MirrorImports::new("#");

/// Path of the mirrors the frontend compiles against: the sources of the `@xrf/ipc` package, beside its runtime.
fn bindings_output_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../packages/xrf-ipc/src")
}

/// Every crate writes its own module, the application's crates (`xrf_app_core`, each plugin) included, so a page
/// imports a type from the crate that owns it.
fn type_modules() -> TypeModules {
  TypeModules::default()
}

/// Rewrite the committed mirrors in place.
#[test]
#[ignore]
fn export_typescript_bindings() {
  IpcBindingsGenerator::generate(
    &bindings_output_path(),
    &command_surfaces::<tauri::Wry>(),
    &type_modules(),
    &MIRROR_IMPORTS,
  );
}

/// Fail only where the committed mirrors no longer describe what the Rust sources produce.
#[test]
fn verify_typescript_bindings() {
  let drift: SurfaceDrift = IpcBindingsGenerator::verify(
    &bindings_output_path(),
    &GeneratedResources::path("bindings"),
    &command_surfaces::<tauri::Wry>(),
    &type_modules(),
    &MIRROR_IMPORTS,
  );

  assert!(!drift.is_breaking(), "{}", drift.describe());
}
