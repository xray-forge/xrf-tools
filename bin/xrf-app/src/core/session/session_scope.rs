use serde::{Deserialize, Serialize};

/// Who holds an opening: a tool in a window, named alike across the window's page loads, so a reloaded page attaches to
/// what its tool held rather than opening it again.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionScope {
  /// The window's label.
  // todo: Read the window from the invoking webview in the `sessions` commands rather than trusting the page's label,
  // before a second window is opened.
  pub window: String,
  /// The tool within the window: its application id, with an instance number where one window runs two.
  pub tool: String,
}
