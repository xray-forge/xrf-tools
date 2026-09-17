use std::path::PathBuf;

/// Directories the application knows about itself, resolved once while the plugin is set up.
pub struct SystemPathsState {
  /// Writable directory belonging to this application, used when its own is not writable.
  pub local_data: Option<PathBuf>,
}

impl SystemPathsState {
  pub fn new(local_data: Option<PathBuf>) -> Self {
    Self { local_data }
  }
}
