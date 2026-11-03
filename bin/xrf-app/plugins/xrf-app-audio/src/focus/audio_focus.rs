use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, PoisonError};

use xrf_audio::AudioServer;

use crate::settings::AudioSettingsStore;

/// Which windows have the focus, and so whether the application plays in the background: where the settings say so,
/// it is silent once no window has the focus.
#[derive(Default)]
pub struct AudioFocus {
  /// The focused windows' labels, or `None` before any window said: the application counts as focused until then, so
  /// it never starts muted, and while any window has the focus, whatever order their events come in.
  focused_windows: Mutex<Option<HashSet<String>>>,
}

impl AudioFocus {
  /// Notes whether the window `label` has the focus, answering whether the application went to or came back from the
  /// background.
  pub fn set_window_focused(&self, label: &str, is_focused: bool) -> bool {
    let mut windows: MutexGuard<'_, Option<HashSet<String>>> = self.lock();
    let was_in_background: bool = Self::is_background(&windows);
    let focused: &mut HashSet<String> = windows.get_or_insert_default();

    if is_focused {
      focused.insert(label.to_owned());
    } else {
      focused.remove(label);
    }

    Self::is_background(&windows) != was_in_background
  }

  /// Whether no window has the focus.
  pub fn is_in_background(&self) -> bool {
    Self::is_background(&self.lock())
  }

  /// Plays at the stored volumes, silent in the background where the settings say so. The focus stays locked while
  /// the gains go, so two windows' or a window's and the settings' changes never land out of order.
  pub fn apply(&self, server: &AudioServer, store: &AudioSettingsStore) {
    let windows: MutexGuard<'_, Option<HashSet<String>>> = self.lock();

    server.set_gains(store.get().get_gains(Self::is_background(&windows)));
  }

  fn is_background(windows: &Option<HashSet<String>>) -> bool {
    windows.as_ref().is_some_and(HashSet::is_empty)
  }

  fn lock(&self) -> MutexGuard<'_, Option<HashSet<String>>> {
    self.focused_windows.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
