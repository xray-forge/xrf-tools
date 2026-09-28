use std::path::PathBuf;
use std::{env, fs, io};

#[path = "src/ipc/registry/build.rs"]
mod registry;

use tauri_build::{Attributes, WindowsAttributes};
use xrf_build_info::BuildKind;

fn main() {
  xrf_build_info::emit();

  clear_stale_inline_plugin_permissions().expect("failed to clear stale inline-plugin permissions");

  println!("cargo:rerun-if-changed=icons");

  tauri_build::try_build(registry::configure(
    Attributes::new()
      .codegen(tauri_build::CodegenContext::new())
      .windows_attributes(resolve_windows_attributes()),
  ))
  .expect("failed to run tauri-build")
}

/// The executable's own icon, which Explorer and shortcuts show, carries the build kind in its core colour.
fn resolve_windows_attributes() -> WindowsAttributes {
  let attributes: WindowsAttributes = WindowsAttributes::new();

  match BuildKind::from_recorded(env::var("XRF_BUILD_KIND").ok().as_deref()) {
    BuildKind::Optimized => attributes,
    BuildKind::Development => attributes.window_icon_path("icons/development/icon.ico"),
    BuildKind::Local => attributes.window_icon_path("icons/local/icon.ico"),
  }
}

// Tauri overwrites current command permissions but does not remove files for commands deleted from inline plugins.
fn clear_stale_inline_plugin_permissions() -> io::Result<()> {
  let out_dir: PathBuf = env::var_os("OUT_DIR")
    .map(PathBuf::from)
    .ok_or_else(|| io::Error::other("Cargo did not provide OUT_DIR"))?;
  let plugins_dir: PathBuf = out_dir.join("plugins");

  match fs::remove_dir_all(&plugins_dir) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(io::Error::new(
      error.kind(),
      format!("Failed to remove {}: {error}", plugins_dir.display()),
    )),
  }
}
