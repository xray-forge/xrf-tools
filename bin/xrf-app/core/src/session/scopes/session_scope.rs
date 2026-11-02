use crate::window::InvokingWindow;

/// Who holds an opening: a tool in a window, named alike across the window's page loads, so a reloaded page attaches to
/// what its tool held rather than opening it again. The window is the one a command was invoked from, never one a page
/// names, so no page can reach another window's openings.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SessionScope {
  /// The window's label.
  pub window: String,
  /// The tool within the window: its application id, with an instance number where one window runs two.
  pub tool: String,
}

impl SessionScope {
  /// The scope of `tool` in the window labelled `window`.
  pub fn new(window: impl Into<String>, tool: impl Into<String>) -> Self {
    Self {
      window: window.into(),
      tool: tool.into(),
    }
  }

  /// The scope of `tool` in `window`, the window a command was invoked from.
  pub fn of(window: &InvokingWindow, tool: impl Into<String>) -> Self {
    Self::new(window.get_label(), tool)
  }
}
