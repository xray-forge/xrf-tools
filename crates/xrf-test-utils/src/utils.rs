use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use fileslice::FileSlice;

/// Get relative path to sample resource.
pub fn build_relative_test_sample_file_path(file: &str, resource: &str) -> String {
  let mut path: PathBuf = PathBuf::new();

  path.push(Path::new(file).file_stem().unwrap());
  path.push(resource);

  path.into_os_string().into_string().unwrap()
}

/// Get relative path to sample resource of current test file.
pub fn build_relative_test_sample_file_directory(file: &str) -> String {
  let mut path: PathBuf = PathBuf::new();

  path.push(Path::new(file).file_stem().unwrap());

  path.into_os_string().into_string().unwrap()
}

/// Get absolute path to a generated test resource.
pub fn build_absolute_generated_test_resource_path(resource_path: &str) -> PathBuf {
  static GENERATED_TEST_RESOURCE_ROOT: OnceLock<PathBuf> = OnceLock::new();

  let root: &PathBuf = GENERATED_TEST_RESOURCE_ROOT.get_or_init(|| {
    let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root: &Path = manifest_dir
      .parent()
      .and_then(Path::parent)
      .expect("test utility crate to be inside the workspace crates directory");
    let executable: String = std::env::current_exe()
      .ok()
      .and_then(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
      .unwrap_or_else(|| String::from("test"));
    let started_at: u128 = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock to be after the Unix epoch")
      .as_nanos();

    workspace_root
      .join("target")
      .join("test-resources")
      .join(format!("{executable}-{}-{started_at}", std::process::id()))
  });

  root.join(resource_path)
}

/// Get absolute path to a generated sample resource.
pub fn build_absolute_generated_test_sample_file_path(file: &str, resource: &str) -> PathBuf {
  build_absolute_generated_test_resource_path(&build_relative_test_sample_file_path(file, resource))
}

/// Open a generated test resource as a slice.
pub fn open_generated_test_resource_as_slice(resource_path: &str) -> IoResult<FileSlice> {
  let path: PathBuf = build_absolute_generated_test_resource_path(resource_path);

  match File::open(path.clone()) {
    Ok(file) => Ok(FileSlice::new(file)),
    Err(error) => Err(IoError::new(
      error.kind(),
      format!("Failed to open generated test asset {}", path.display()),
    )),
  }
}

/// Open a generated test resource.
pub fn open_generated_test_resource_as_file(resource_path: &str) -> IoResult<File> {
  let path: PathBuf = build_absolute_generated_test_resource_path(resource_path);

  match File::open(path.clone()) {
    Ok(file) => Ok(file),
    Err(error) => Err(IoError::new(
      error.kind(),
      format!("Failed to open generated test asset {}", path.display()),
    )),
  }
}

/// Create and open a generated test resource, overwriting any previous output.
pub fn overwrite_generated_test_resource_as_file(resource_path: &str) -> IoResult<File> {
  let path: PathBuf = build_absolute_generated_test_resource_path(resource_path);

  std::fs::create_dir_all(path.parent().expect("Parent directory"))?;

  match OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .read(true)
    .open(path.clone())
  {
    Ok(file) => Ok(file),
    Err(error) => Err(IoError::new(
      error.kind(),
      format!("Failed to open generated test asset {}", path.display()),
    )),
  }
}

/// Write a generated test resource and return where it landed.
///
/// The common shape of a test that needs a file on disk: create it, fill it, close it, and get the
/// path back to hand to the code under test. Closing before returning matters - a reader opening a
/// file this still held would be racing the handle on Windows.
///
/// Takes anything byte-like, so a test can pass a string literal or an already-encoded buffer.
pub fn write_generated_test_resource<C: AsRef<[u8]>>(resource_path: &str, contents: C) -> IoResult<PathBuf> {
  let mut file: File = overwrite_generated_test_resource_as_file(resource_path)?;

  file.write_all(contents.as_ref())?;
  drop(file);

  Ok(build_absolute_generated_test_resource_path(resource_path))
}

/// Create and open file by path, overwrite existing one.
pub fn overwrite_file<P: AsRef<Path>>(path: P) -> IoResult<File> {
  std::fs::create_dir_all(path.as_ref().parent().expect("Parent directory"))?;

  match OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .read(true)
    .open(path.as_ref())
  {
    Ok(file) => Ok(file),
    Err(error) => Err(IoError::new(
      error.kind(),
      format!("Failed to open test asset {}", path.as_ref().display()),
    )),
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;

  #[test]
  fn writes_a_resource_and_reports_where_it_landed() -> IoResult<()> {
    let path: PathBuf = write_generated_test_resource("utils/written.txt", "contents")?;

    assert_eq!(fs::read(&path)?, b"contents");
    assert_eq!(path, build_absolute_generated_test_resource_path("utils/written.txt"));

    Ok(())
  }

  #[test]
  fn writes_bytes_as_readily_as_text() -> IoResult<()> {
    let path: PathBuf = write_generated_test_resource("utils/written.bin", [0xEF, 0xBB, 0xBF])?;

    assert_eq!(fs::read(path)?, vec![0xEF, 0xBB, 0xBF]);

    Ok(())
  }

  #[test]
  fn replaces_whatever_was_there_before() -> IoResult<()> {
    write_generated_test_resource("utils/replaced.txt", "first")?;

    let path: PathBuf = write_generated_test_resource("utils/replaced.txt", "second")?;

    // Truncating rather than appending, so a rerun does not read the previous run's tail.
    assert_eq!(fs::read(path)?, b"second");

    Ok(())
  }

  #[test]
  fn creates_the_directories_above_the_resource() -> IoResult<()> {
    let path: PathBuf = write_generated_test_resource("utils/nested/deeper/file.txt", "x")?;

    assert!(path.is_file());

    Ok(())
  }

  #[test]
  fn generated_resources_stay_under_target() {
    let path: PathBuf = build_absolute_generated_test_resource_path("utils/generated.bin");
    let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root: &Path = manifest_dir
      .parent()
      .and_then(Path::parent)
      .expect("test utility crate to be inside the workspace crates directory");

    assert!(path.starts_with(workspace_root.join("target").join("test-resources")));
  }

  #[test]
  fn reads_back_what_the_overwrite_handle_wrote() -> IoResult<()> {
    let resource: &str = "utils/handle.bin";
    let mut generated_file: File = overwrite_generated_test_resource_as_file(resource)?;

    generated_file.write_all(b"generated")?;
    drop(generated_file);

    assert_eq!(
      fs::read(build_absolute_generated_test_resource_path(resource))?,
      b"generated"
    );
    assert_eq!(
      open_generated_test_resource_as_slice(resource)?.bytes_remaining(),
      b"generated".len()
    );
    assert_eq!(
      open_generated_test_resource_as_file(resource)?.metadata()?.len(),
      b"generated".len() as u64
    );

    Ok(())
  }
}
