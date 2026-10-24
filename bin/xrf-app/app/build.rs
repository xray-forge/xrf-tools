use std::path::PathBuf;
use std::{env, fs, io};

// Each plugin crate's command domains, which the inline plugins and their permissions are declared from: the macro
// alone, as the build script compiles none of the crates.
#[macro_use]
#[path = "../core/src/registry/domains.rs"]
mod core_domains;
#[macro_use]
#[path = "../plugins/xrf-app-render/src/registry/domains.rs"]
mod render_domains;
#[macro_use]
#[path = "../plugins/xrf-app-archives/src/registry/domains.rs"]
mod archives_domains;
#[macro_use]
#[path = "../plugins/xrf-app-configs/src/registry/domains.rs"]
mod configs_domains;
#[macro_use]
#[path = "../plugins/xrf-app-dialogs/src/registry/domains.rs"]
mod dialogs_domains;
#[macro_use]
#[path = "../plugins/xrf-app-exports/src/registry/domains.rs"]
mod exports_domains;
#[macro_use]
#[path = "../plugins/xrf-app-gamedata/src/registry/domains.rs"]
mod gamedata_domains;
#[macro_use]
#[path = "../plugins/xrf-app-levels/src/registry/domains.rs"]
mod levels_domains;
#[macro_use]
#[path = "../plugins/xrf-app-spawn/src/registry/domains.rs"]
mod spawn_domains;
#[macro_use]
#[path = "../plugins/xrf-app-sprite-equipment/src/registry/domains.rs"]
mod sprite_equipment_domains;
#[macro_use]
#[path = "../plugins/xrf-app-system/src/registry/domains.rs"]
mod system_domains;
#[macro_use]
#[path = "../plugins/xrf-app-textures/src/registry/domains.rs"]
mod textures_domains;
#[macro_use]
#[path = "../plugins/xrf-app-translations/src/registry/domains.rs"]
mod translations_domains;
#[macro_use]
#[path = "../plugins/xrf-app-visuals/src/registry/domains.rs"]
mod visuals_domains;

#[path = "src/ipc/registry/build.rs"]
mod registry;

use tauri_build::{Attributes, WindowsAttributes};
use xrf_build_info::BuildKind;

fn main() {
  xrf_build_info::emit();

  clear_stale_inline_plugin_permissions().expect("failed to clear stale inline-plugin permissions");

  println!("cargo:rerun-if-changed=icons");

  link_dlss_runtime();
  add_dlss_resources();

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

/// Tauri links the C runtime's vcruntime statically, and NGX's library imports `wcsstr` from the DLL one: the linker
/// binds an import only to a definition already in the link, so libvcruntime's is pulled in, and the notes that each of
/// its imports is bound to a static definition are expected.
fn link_dlss_runtime() {
  let is_msvc: bool = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");

  if is_msvc && env::var_os("CARGO_FEATURE_DLSS").is_some() {
    println!("cargo:rustc-link-arg-bins=/INCLUDE:wcsstr");
    println!("cargo:rustc-link-arg-bins=/IGNORE:4217");
  }
}

/// Built with DLSS, its runtime library joins the renderer's bundle as `tauri.dlss.conf.json` maps it, which
/// tauri-build copies into the profile's resources: merged here unless the Tauri CLI already merged a configuration.
fn add_dlss_resources() {
  println!("cargo:rerun-if-changed=tauri.dlss.conf.json");

  if env::var_os("CARGO_FEATURE_DLSS").is_none() || env::var_os("TAURI_CONFIG").is_some() {
    return;
  }

  let config: String = fs::read_to_string("tauri.dlss.conf.json").expect("tauri.dlss.conf.json is readable");

  // SAFETY: the build script runs on one thread, and tauri-build reads the variable after this.
  unsafe { env::set_var("TAURI_CONFIG", config) };
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
