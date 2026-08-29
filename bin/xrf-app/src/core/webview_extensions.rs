use std::path::PathBuf;

use tauri::webview::WebviewWindowBuilder;
use tauri::{Manager, Runtime};
use xrf_utils::format_path;

/// Browser extensions a developer supplied locally, loaded into the application window.
pub trait DevExtensions {
  /// Install every unpacked extension the local extensions directory holds.
  ///
  /// Leaves the builder untouched in release builds, outside Windows, and whenever the directory is
  /// absent, empty, or holds anything WebView2 would reject.
  fn with_dev_extensions(self) -> Self;
}

impl<R: Runtime, M: Manager<R>> DevExtensions for WebviewWindowBuilder<'_, R, M> {
  fn with_dev_extensions(self) -> Self {
    match resolve_extensions_directory() {
      Some(directory) => {
        log::info!("Loading webview extensions from '{}'", format_path(&directory));

        self.browser_extensions_enabled(true).extensions_path(directory)
      }
      None => self,
    }
  }
}

/// Resolve the directory unpacked extensions are installed from, when it holds only valid ones.
///
/// Chrome extensions are a WebView2 capability, so this only ever resolves on Windows: elsewhere
/// `extensions_path` means compiled `.so` web process extensions and would be handed the wrong
/// payload. Requires WebView2 Runtime `120.0.2210.55` or newer, older runtimes ignore extensions.
#[cfg(all(debug_assertions, windows))]
fn resolve_extensions_directory() -> Option<PathBuf> {
  /// Absolute path of an alternative directory to load unpacked extensions from.
  const DIRECTORY_ENV: &str = "XRF_APP_EXTENSIONS_DIR";

  let directory: PathBuf = match std::env::var_os(DIRECTORY_ENV) {
    Some(path) => PathBuf::from(path),
    None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("extensions")
      .join("unpacked"),
  };

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

/// Resolve nothing, on the targets and profiles that cannot load browser extensions.
#[cfg(not(all(debug_assertions, windows)))]
fn resolve_extensions_directory() -> Option<PathBuf> {
  None
}
