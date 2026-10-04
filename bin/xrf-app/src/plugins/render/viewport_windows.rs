use std::collections::HashMap;

use xrf_renderer::RenderViewportId;

/// The window each attached viewport draws into, by the label its page named.
#[derive(Default)]
pub struct ViewportWindows {
  windows: HashMap<RenderViewportId, String>,
}

impl ViewportWindows {
  pub fn insert(&mut self, viewport: RenderViewportId, window: &str) {
    self.windows.insert(viewport, window.to_string());
  }

  /// Forgets a viewport, answering the window it drew into, if any held it.
  pub fn remove(&mut self, viewport: RenderViewportId) -> Option<String> {
    self.windows.remove(&viewport)
  }

  /// Forgets every viewport drawn into a window, answering them in the order they were attached.
  pub fn take_window(&mut self, window: &str) -> Vec<RenderViewportId> {
    let mut taken: Vec<RenderViewportId> = Vec::new();

    self.windows.retain(|viewport, label| {
      let is_taken: bool = label == window;

      if is_taken {
        taken.push(*viewport);
      }

      !is_taken
    });

    taken.sort();

    taken
  }
}
