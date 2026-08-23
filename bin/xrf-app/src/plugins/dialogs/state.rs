use std::sync::{Arc, Mutex};

use xrf_dialog::DialogProject;

/// The open dialog project.
///
/// Holds the parsed project rather than only its descriptor, because the phrases and their source
/// ranges are what later reads and edits work from, and re-reading the tree per selection would hand
/// out ranges into a string nobody still holds.
pub struct DialogProjectState {
  pub project: Arc<Mutex<Option<DialogProject>>>,
}

impl DialogProjectState {
  pub fn new() -> Self {
    Self {
      project: Arc::new(Mutex::new(None)),
    }
  }
}
