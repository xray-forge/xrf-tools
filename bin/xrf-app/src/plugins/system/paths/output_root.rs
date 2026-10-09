use std::path::PathBuf;

use crate::core::directories::{get_executable_directory, is_writable};
use crate::core::types::TauriResult;

/// Directory name held beside the application, or in its data directory, for everything tools write.
const OUTPUT_DIRECTORY: &str = "target";

/// Where tools write when no output directory has been configured.
pub fn default_output_root(local_data: Option<PathBuf>) -> TauriResult<String> {
  select_default_output_root(
    get_executable_directory().filter(|directory| is_writable(directory)),
    local_data,
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
