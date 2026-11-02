//! Loading unpacked browser extensions a developer supplied locally into the application window.
//!
//! Only a Windows debug build resolves anything: Chrome extensions are a WebView2 capability, and on other targets
//! `extensions_path` means compiled `.so` web process extensions, which would be handed the wrong payload. Requires
//! WebView2 Runtime `120.0.2210.55` or newer; older runtimes ignore extensions silently.

use std::path::PathBuf;

use tauri::webview::WebviewWindowBuilder;
use tauri::{Manager, Runtime};
use xrf_utils::format_path;

/// Browser extensions a developer supplied locally, loaded into the application window.
pub struct WebviewExtensions;

impl WebviewExtensions {
  /// Installs every unpacked extension the local extensions directory holds.
  pub fn apply<R: Runtime, M: Manager<R>>(builder: WebviewWindowBuilder<R, M>) -> WebviewWindowBuilder<R, M> {
    match Self::resolve_directory() {
      Some(directory) => {
        log::info!("Loading webview extensions from '{}'", format_path(&directory));

        builder.browser_extensions_enabled(true).extensions_path(directory)
      }
      None => builder,
    }
  }

  /// The directory unpacked extensions are installed from, when it holds only valid ones.
  #[cfg(all(debug_assertions, windows))]
  fn resolve_directory() -> Option<PathBuf> {
    let directory: PathBuf = xrf_env_flags::EnvFlags::APP_EXTENSIONS_DIR
      .read_path()
      .unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("extensions")
          .join("unpacked")
      });

    let entries: Vec<PathBuf> = std::fs::read_dir(&directory)
      .ok()?
      .flatten()
      .map(|entry| entry.path())
      .collect();

    // WebView2 rejects an entry that is not the topmost folder of an unpacked extension, and wry lets
    // that failure abort window creation, so one stray file costs the extensions and not the app.
    for entry in &entries {
      if !entry.join("manifest.json").is_file() {
        log::error!(
          "Skipping webview extensions, '{}' has no manifest.json and WebView2 would reject it",
          format_path(entry)
        );

        return None;
      }
    }

    (!entries.is_empty()).then_some(directory)
  }

  /// Nothing, on the targets and profiles that cannot load browser extensions.
  #[cfg(not(all(debug_assertions, windows)))]
  fn resolve_directory() -> Option<PathBuf> {
    None
  }
}
