use std::collections::HashMap;
use std::fmt::Display;

/// Failures a frame meets again every frame until something changes, logged when one starts or changes and once when
/// it ends, rather than at every frame's rate.
#[derive(Default)]
pub struct FrameFailureLog {
  /// The failure last logged for each thing that failed, by what it is.
  failing: HashMap<&'static str, String>,
}

impl FrameFailureLog {
  /// Logs `error` for `what`, unless it is the failure last logged for it.
  pub fn fail(&mut self, what: &'static str, error: &impl Display) {
    let text: String = error.to_string();

    if self.failing.get(what) != Some(&text) {
      log::error!("{what}: {text}");
      self.failing.insert(what, text);
    }
  }

  /// Notes that `what` succeeded, logging that it recovered where it was failing.
  pub fn succeed(&mut self, what: &'static str) {
    if self.failing.remove(what).is_some() {
      log::info!("{what}: recovered");
    }
  }
}
