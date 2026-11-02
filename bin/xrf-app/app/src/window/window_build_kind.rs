use tauri::image::Image;
use tauri::webview::WebviewWindowBuilder;
use tauri::{Manager, Runtime, include_image};
use xrf_build_info::BuildKind;

/// What the main window shows of the kind of build it belongs to.
pub struct WindowBuildKind;

impl WindowBuildKind {
  /// Carries `kind` into the window's icon, and into the document before any of its scripts run.
  ///
  /// # Errors
  ///
  /// Returns an error when the icon cannot be set.
  pub fn apply<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
    kind: BuildKind,
  ) -> tauri::Result<WebviewWindowBuilder<'a, R, M>> {
    let builder: WebviewWindowBuilder<'a, R, M> = builder.initialization_script(Self::get_script(kind));

    match Self::resolve_icon(kind) {
      Some(icon) => builder.icon(icon),
      None => Ok(builder),
    }
  }

  /// Script recording `kind` where the document reads it, as `window.__XRF_BUILD_KIND__`.
  pub fn get_script(kind: BuildKind) -> String {
    format!(
      "Object.defineProperty(window, \"__XRF_BUILD_KIND__\", {{ value: \"{}\" }});",
      kind.as_str()
    )
  }

  /// The running window's icon for `kind`, or `None` for the one the configuration already embeds.
  ///
  /// Tauri embeds the first ICO entry, which these icons keep at 48 px, and a PNG where there is no ICO.
  fn resolve_icon(kind: BuildKind) -> Option<Image<'static>> {
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
}
