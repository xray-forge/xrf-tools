use tauri::image::Image;
use tauri::webview::WebviewWindowBuilder;
use tauri::{Manager, Runtime, include_image};
use xrf_build_info::BuildKind;

/// What the main window shows of the kind of build it belongs to.
pub trait WindowBuildKind: Sized {
  /// Carry `kind` into the window's icon, and into the document before any of its scripts run.
  fn with_build_kind(self, kind: BuildKind) -> tauri::Result<Self>;
}

impl<R: Runtime, M: Manager<R>> WindowBuildKind for WebviewWindowBuilder<'_, R, M> {
  fn with_build_kind(self, kind: BuildKind) -> tauri::Result<Self> {
    let builder: Self = self.initialization_script(get_build_kind_script(kind));

    match resolve_window_icon(kind) {
      Some(icon) => builder.icon(icon),
      None => Ok(builder),
    }
  }
}

/// Script recording `kind` where the document reads it, as `window.__XRF_BUILD_KIND__`.
pub fn get_build_kind_script(kind: BuildKind) -> String {
  format!(
    "Object.defineProperty(window, \"__XRF_BUILD_KIND__\", {{ value: \"{}\" }});",
    kind.as_str()
  )
}

/// The running window's icon for `kind`, or `None` for the one the configuration already embeds.
///
/// Tauri embeds the first ICO entry, which these icons keep at 48 px, and a PNG where there is no ICO.
fn resolve_window_icon(kind: BuildKind) -> Option<Image<'static>> {
  match kind {
    BuildKind::Optimized => None,
    #[cfg(windows)]
    BuildKind::Development => Some(include_image!("icons/development/icon.ico")),
    #[cfg(windows)]
    BuildKind::Local => Some(include_image!("icons/local/icon.ico")),
    #[cfg(not(windows))]
    BuildKind::Development => Some(include_image!("icons/development/32x32.png")),
    #[cfg(not(windows))]
    BuildKind::Local => Some(include_image!("icons/local/32x32.png")),
  }
}
