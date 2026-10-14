//! Where the application may write: beside its executable when that directory accepts files, as a portable install
//! keeps everything together, else in its own data directory.

use std::path::{Path, PathBuf};
use std::{env, fs};

/// The directory the running executable lies in.
pub fn get_executable_directory() -> Option<PathBuf> {
  env::current_exe()
    .ok()
    .and_then(|executable| executable.parent().map(Path::to_path_buf))
}

/// The application's local data directory, as Tauri resolves it for the bundle `identifier`, known before Tauri runs.
pub fn get_local_data_directory(identifier: &str) -> Option<PathBuf> {
  dirs::data_local_dir().map(|directory| directory.join(identifier))
}

/// Whether a directory accepts a file, tested by writing one and removing it again.
pub fn is_writable(directory: &Path) -> bool {
  let probe: PathBuf = directory.join(".xrf-write-probe");

  match fs::write(&probe, []) {
    Ok(()) => {
      let _ = fs::remove_file(&probe);

      true
    }
    Err(_) => false,
  }
}
