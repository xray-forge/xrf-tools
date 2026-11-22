use log::Level;

/// One record the webview logged, as [`crate::logging::UiLog::write`] takes it.
pub struct UiLogEntry<'a> {
  pub level: Level,
  /// The webview's scope, written under the target `xrf_ui::<scope>`; none writes under `xrf_ui` itself.
  pub scope: &'a str,
  /// Milliseconds since the Unix epoch at which the webview logged it, which its line is stamped with.
  pub time: u64,
  /// The text, a stack's many lines included.
  pub message: &'a str,
}
