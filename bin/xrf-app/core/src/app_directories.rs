use std::path::PathBuf;

use crate::directories::{get_app_data_directories, get_output_directories};

/// Where this application writes, decided from its local data directory as it starts.
pub struct AppDirectories {
  /// Writable directory belonging to this application, used when its own is not writable.
  local_data: Option<PathBuf>,
}

impl AppDirectories {
  pub fn new(local_data: Option<PathBuf>) -> Self {
    Self { local_data }
  }

  /// Where the application keeps its own files, best first.
  pub fn get_app_data_directories(&self) -> Vec<PathBuf> {
    get_app_data_directories(self.local_data.clone())
  }

  /// Where tools write when no output directory is configured, best first.
  pub fn get_output_directories(&self) -> Vec<PathBuf> {
    get_output_directories(self.local_data.clone())
  }
}
