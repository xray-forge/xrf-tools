use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, PoisonError};

use xrf_audio::AudioServer;

use crate::settings::AudioSettingsStore;

/// Which windows have the focus, which decides the background mute the settings may ask for.
#[derive(Default)]
pub struct AudioFocus {
  /// The focused windows' labels, or `None` before any window reported: the application counts as focused until then,
  /// so it never starts muted, and while any window has the focus, in whatever order their events come.
  focused_windows: Mutex<Option<HashSet<String>>>,
}

impl AudioFocus {
  /// Notes the focus of window `label`; returns whether the application entered or left the background.
  pub fn set_window_focused(&self, label: &str, is_focused: bool) -> bool {
    let mut windows: MutexGuard<'_, Option<HashSet<String>>> = self.lock();
    let was_in_background: bool = Self::is_background(windows.as_ref());
    let focused: &mut HashSet<String> = windows.get_or_insert_default();

    if is_focused {
      focused.insert(label.to_owned());
    } else {
      focused.remove(label);
    }

    Self::is_background(windows.as_ref()) != was_in_background
  }

  /// Whether no window has the focus.
  pub fn is_in_background(&self) -> bool {
    Self::is_background(self.lock().as_ref())
  }

  /// Sets the server's gains from the stored volumes and the background mute.
  ///
  /// The focus stays locked while the gains are sent, so concurrent focus and settings changes never land out of order.
  pub fn apply(&self, server: &AudioServer, store: &AudioSettingsStore) {
    let windows: MutexGuard<'_, Option<HashSet<String>>> = self.lock();

    server.set_gains(store.get().get_gains(Self::is_background(windows.as_ref())));
  }

  fn is_background(windows: Option<&HashSet<String>>) -> bool {
    windows.is_some_and(HashSet::is_empty)
  }

  fn lock(&self) -> MutexGuard<'_, Option<HashSet<String>>> {
    self.focused_windows.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
