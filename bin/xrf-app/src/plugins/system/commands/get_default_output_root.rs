use std::path::{Path, PathBuf};
use std::{env, fs};

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::system::state::SystemPathsState;

/// Directory name held beside the application, or in its data directory, for everything tools write.
const OUTPUT_DIRECTORY: &str = "target";

/// Where tools write when no output directory has been configured.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_default_output_root"))]
#[tauri::command(rename = "get_default_output_root")]
pub async fn system_get_default_output_root(state: State<'_, SystemPathsState>) -> TauriResult<String> {
  select_default_output_root(
    executable_directory().filter(|directory| is_writable(directory)),
    state.local_data.clone(),
  )
}

/// Selects the preferred output root that the JavaScript caller can address without changing it.
fn select_default_output_root(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> TauriResult<String> {
  executable
    .and_then(to_wire_output_root)
    .or_else(|| local_data.and_then(to_wire_output_root))
    .ok_or_else(|| String::from("No available output root has a Unicode path"))
}

fn to_wire_output_root(root: PathBuf) -> Option<String> {
  root.join(OUTPUT_DIRECTORY).into_os_string().into_string().ok()
}

fn executable_directory() -> Option<PathBuf> {
  env::current_exe()
    .ok()
    .and_then(|executable| executable.parent().map(Path::to_path_buf))
}

/// Whether a directory accepts a file, tested by writing one and removing it again.
///
/// Metadata cannot answer this on Windows, where a directory under an installation root reports itself writable and
/// still refuses the write.
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

#[cfg(test)]
mod tests {
  #[cfg(any(unix, windows))]
  use std::ffi::OsString;
  use std::path::PathBuf;

  use super::select_default_output_root;

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
  fn appends_the_output_directory_to_the_preferred_root() {
    assert_eq!(
      select_default_output_root(Some(PathBuf::from("application")), Some(PathBuf::from("data")))
        .expect("default output root"),
      PathBuf::from("application").join("target").display().to_string()
    );
  }

  #[cfg(any(unix, windows))]
  #[test]
  fn falls_back_instead_of_rendering_a_non_unicode_root_lossily() {
    assert_eq!(
      select_default_output_root(
        Some(PathBuf::from(non_unicode_component())),
        Some(PathBuf::from("data")),
      )
      .expect("fallback output root"),
      PathBuf::from("data").join("target").display().to_string()
    );
  }

  #[cfg(any(unix, windows))]
  #[test]
  fn refuses_output_roots_that_cannot_cross_the_wire() {
    let invalid: PathBuf = PathBuf::from(non_unicode_component());

    assert!(select_default_output_root(Some(invalid.clone()), Some(invalid)).is_err());
  }
}
