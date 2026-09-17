/// Every preference the backend keeps between runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreferenceKey {
  /// Where the main window was last left.
  WindowGeometry,
}

impl PreferenceKey {
  /// The one spelling the file is written with.
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::WindowGeometry => "window.geometry",
    }
  }
}
