//! Where the application may write: beside its executable when that directory accepts files, as a portable install
//! keeps everything together, else in its own data directory. Each kind of place is a list of candidates, best first.

use std::path::{Path, PathBuf};
use std::{env, fs};

/// Directory beside the executable for what the application keeps of its own, as X-Ray keeps its `appdata`: logs, and
/// files a tool keeps for its user.
pub const APP_DATA_DIRECTORY: &str = "appdata";

/// Directory, beside the executable or in the local data directory, for everything tools write.
pub const OUTPUT_DIRECTORY: &str = "target";

/// The directory the running executable lies in.
fn get_executable_directory() -> Option<PathBuf> {
  env::current_exe()
    .ok()
    .and_then(|executable| executable.parent().map(Path::to_path_buf))
}

/// The application's local data directory, as Tauri resolves it for the bundle `identifier`, known before Tauri runs.
pub fn get_local_data_directory(identifier: &str) -> Option<PathBuf> {
  dirs::data_local_dir().map(|directory| directory.join(identifier))
}

/// Where the application keeps its own files, best first: `appdata` beside the executable where that accepts files,
/// then its local data directory `local_data`.
pub fn get_app_data_directories(local_data: Option<PathBuf>) -> Vec<PathBuf> {
  get_ordered_app_data_directories(get_writable_executable_directory(), local_data)
}

/// Where tools write when no output directory is configured, best first: `target` beside the executable where that
/// accepts files, then in the local data directory `local_data`.
pub fn get_output_directories(local_data: Option<PathBuf>) -> Vec<PathBuf> {
  get_ordered_output_directories(get_writable_executable_directory(), local_data)
}

/// The first of `directories` the JavaScript side can address without changing it: one with a Unicode path.
pub fn get_first_unicode_path(directories: Vec<PathBuf>) -> Option<String> {
  directories
    .into_iter()
    .find_map(|directory| directory.into_os_string().into_string().ok())
}

/// Whether a directory accepts a file, tested by writing one and removing it again.
fn is_writable(directory: &Path) -> bool {
  let probe: PathBuf = directory.join(".xrf-write-probe");

  match fs::write(&probe, []) {
    Ok(()) => {
      let _ = fs::remove_file(&probe);

      true
    }
    Err(_) => false,
  }
}

/// The directory the running executable lies in, where it accepts files.
fn get_writable_executable_directory() -> Option<PathBuf> {
  get_executable_directory().filter(|directory| is_writable(directory))
}

fn get_ordered_app_data_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
  executable
    .map(|directory| directory.join(APP_DATA_DIRECTORY))
    .into_iter()
    .chain(local_data)
    .collect()
}

fn get_ordered_output_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
  executable
    .into_iter()
    .chain(local_data)
    .map(|directory| directory.join(OUTPUT_DIRECTORY))
    .collect()
}

#[cfg(test)]
mod tests {
  #[cfg(any(unix, windows))]
  use std::ffi::OsString;
  use std::path::PathBuf;

  use super::{get_first_unicode_path, get_ordered_app_data_directories, get_ordered_output_directories};

  #[cfg(unix)]
  fn non_unicode_component() -> OsString {
    use std::os::unix::ffi::OsStringExt;

    OsString::from_vec(b"root-\xff".to_vec())
  }

  #[cfg(windows)]
  fn non_unicode_component() -> OsString {
    use std::os::windows::ffi::OsStringExt;

    let mut name: Vec<u16> = "root-".encode_utf16().collect();
    name.push(0xd800);

    OsString::from_wide(&name)
  }

  #[test]
  fn keeps_its_files_beside_the_executable_first() {
    assert_eq!(
      get_ordered_app_data_directories(Some(PathBuf::from("application")), Some(PathBuf::from("data"))),
      vec![PathBuf::from("application").join("appdata"), PathBuf::from("data")]
    );
  }

  #[test]
  fn falls_back_to_its_data_directory_where_the_executable_takes_no_files() {
    assert_eq!(
      get_ordered_app_data_directories(None, Some(PathBuf::from("data"))),
      vec![PathBuf::from("data")]
    );
    assert!(get_ordered_app_data_directories(None, None).is_empty());
  }

  #[test]
  fn writes_tools_output_beside_the_executable_first() {
    assert_eq!(
      get_ordered_output_directories(Some(PathBuf::from("application")), Some(PathBuf::from("data"))),
      vec![
        PathBuf::from("application").join("target"),
        PathBuf::from("data").join("target")
      ]
    );
  }

  #[cfg(any(unix, windows))]
  #[test]
  fn passes_over_a_directory_that_cannot_cross_the_wire_unchanged() {
    let invalid: PathBuf = PathBuf::from(non_unicode_component());

    assert_eq!(
      get_first_unicode_path(get_ordered_output_directories(
        Some(invalid.clone()),
        Some(PathBuf::from("data"))
      )),
      Some(PathBuf::from("data").join("target").display().to_string())
    );
    assert_eq!(get_first_unicode_path(vec![invalid.clone(), invalid]), None);
  }
}
