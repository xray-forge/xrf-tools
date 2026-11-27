#[cfg(any(unix, windows))]
use std::ffi::OsString;
use std::path::PathBuf;

use crate::directories::AppDirectories;

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
fn writes_tools_output_beside_the_executable_first() {
  assert_eq!(
    AppDirectories::order_output_directories(Some(PathBuf::from("application")), Some(PathBuf::from("data"))),
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
    AppDirectories::find_first_unicode_path(AppDirectories::order_output_directories(
      Some(invalid.clone()),
      Some(PathBuf::from("data"))
    )),
    Some(PathBuf::from("data").join("target").display().to_string())
  );
  assert_eq!(
    AppDirectories::find_first_unicode_path(vec![invalid.clone(), invalid]),
    None
  );
}

#[test]
fn keeps_preferences_and_caches_where_it_keeps_its_logs() {
  let directories: AppDirectories = AppDirectories::new(Some(PathBuf::from("data")));
  let own: PathBuf = directories
    .get_app_data_directories()
    .into_iter()
    .next()
    .expect("an own data directory");

  assert_eq!(directories.get_preferences_file(), Some(own.join("preferences.json")));
  assert_eq!(directories.get_cache_root(), Some(own.join("cache")));
  assert_eq!(directories.get_logs_directories().first(), Some(&own.join("logs")));
}
