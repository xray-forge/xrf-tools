use std::collections::HashMap;

use xrf_renderer::{RenderHoldId, RenderViewportId};

/// The window each attached viewport draws into, and each hold was taken for, by the label its page named.
#[derive(Default)]
pub(crate) struct ViewportWindows {
  viewports: HashMap<RenderViewportId, String>,
  holds: HashMap<RenderHoldId, String>,
}

impl ViewportWindows {
  /// Records that a viewport draws into a window.
  pub(crate) fn insert(&mut self, viewport: RenderViewportId, window: &str) {
    self.viewports.insert(viewport, window.to_string());
  }

  /// Forgets a viewport, answering the window it drew into, if any held it.
  pub(crate) fn remove(&mut self, viewport: RenderViewportId) -> Option<String> {
    self.viewports.remove(&viewport)
  }

  /// Records that a window's page holds the renderer.
  pub(crate) fn insert_hold(&mut self, hold: RenderHoldId, window: &str) {
    self.holds.insert(hold, window.to_string());
  }

  /// Forgets a hold, answering the window it was taken for, if any held it.
  pub(crate) fn remove_hold(&mut self, hold: RenderHoldId) -> Option<String> {
    self.holds.remove(&hold)
  }

  /// Forgets every viewport drawn into a window, answering them in the order they were attached.
  pub(crate) fn take_window(&mut self, window: &str) -> Vec<RenderViewportId> {
    Self::take_where(&mut self.viewports, |label| label == window)
  }

  /// Forgets every hold a window's page took, answering them in the order they were taken.
  pub(crate) fn take_window_holds(&mut self, window: &str) -> Vec<RenderHoldId> {
    Self::take_where(&mut self.holds, |label| label == window)
  }

  /// Forgets every viewport and hold, answering each in the order it was attached or taken.
  pub(crate) fn take_all(&mut self) -> (Vec<RenderViewportId>, Vec<RenderHoldId>) {
    (
      Self::take_where(&mut self.viewports, |_| true),
      Self::take_where(&mut self.holds, |_| true),
    )
  }

  fn take_where<K: Copy + Ord>(entries: &mut HashMap<K, String>, is_taken: impl Fn(&str) -> bool) -> Vec<K> {
    let mut taken: Vec<K> = Vec::new();

    entries.retain(|key, label| {
      let is_taken: bool = is_taken(label);

      if is_taken {
        taken.push(*key);
      }

      !is_taken
    });

    taken.sort();

    taken
  }
}
