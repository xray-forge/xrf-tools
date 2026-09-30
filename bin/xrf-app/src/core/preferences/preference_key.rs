/// Every preference the backend keeps between runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreferenceKey {
  /// Where the main window was last left.
  WindowGeometry,
  /// Which optional browser capabilities the webview is started with.
  WebviewOptions,
}

impl PreferenceKey {
  /// The one spelling the file is written with.
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::WindowGeometry => "window.geometry",
      Self::WebviewOptions => "webview.options",
    }
  }
}
