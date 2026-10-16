use std::path::PathBuf;

use crate::contract::render_libraries::RenderLibraries;

#[cfg(windows)]
#[test]
fn reads_a_librarys_file_version() {
  let system: PathBuf = PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()));
  let version: String =
    RenderLibraries::read_file_version(&system.join("System32").join("kernel32.dll")).expect("kernel32 has a version");

  assert_eq!(version.split('.').count(), 4, "{version}");
  assert!(version.split('.').all(|part| part.parse::<u32>().is_ok()), "{version}");
  assert_eq!(
    RenderLibraries::read_file_version(&system.join("no-such-library.dll")),
    None
  );
}
