use std::collections::HashMap;
use std::num::NonZeroIsize;
use std::sync::RwLock;

/// The native handle of every window that can host a native viewport, by its label, recorded as each is built.
///
/// Commands are not generic over the runtime, so a command cannot take the calling window itself; it names the window
/// by label and finds its handle here.
#[derive(Default)]
pub struct WindowHandles {
  handles: RwLock<HashMap<String, NonZeroIsize>>,
}

impl WindowHandles {
  pub fn register(&self, label: &str, handle: NonZeroIsize) {
    self
      .handles
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .insert(label.to_string(), handle);
  }

  pub fn get(&self, label: &str) -> Option<NonZeroIsize> {
    self
      .handles
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .get(label)
      .copied()
  }
}
