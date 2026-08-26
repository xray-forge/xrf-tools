use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fileslice::FileSlice;

/// How long a run's directory survives before a later run collects it.
const GENERATED_RESOURCE_RETENTION: Duration = Duration::from_secs(60 * 60);

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

/// Remove the run directories under `root` that were started before `collect_before`.
///
/// Only entries shaped `<target>-<pid>-<nanos>` are considered, so anything else left in the tree by hand is not this
/// function's to delete. Best effort throughout - a directory another user owns, or one held open on Windows, is left
/// where it is rather than failing a test run over housekeeping.
fn collect_stale_generated_resources(root: &Path, collect_before: u128) {
  let Ok(entries) = std::fs::read_dir(root) else {
    return;
  };

  for entry in entries.flatten() {
    let name: String = entry.file_name().to_string_lossy().into_owned();
    let mut parts = name.rsplitn(3, '-');

    let started_at: Option<u128> = parts.next().and_then(|part| part.parse().ok());
    let process: Option<u32> = parts.next().and_then(|part| part.parse().ok());

    if let (Some(started_at), Some(_), Some(_)) = (started_at, process, parts.next())
      && started_at < collect_before
      && entry.path().is_dir()
    {
      let _ = std::fs::remove_dir_all(entry.path());
    }
  }
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

    // Cargo names a test binary `<target>-<16 hex>`, and that hash moves whenever the toolchain or a dependency does.
    // Dropping it keeps a directory readable and groups every run of one target under the same prefix.
    let target: &str = executable
      .rsplit_once('-')
      .filter(|(_, hash)| hash.len() == 16 && hash.chars().all(|char| char.is_ascii_hexdigit()))
      .map_or(executable.as_str(), |(name, _)| name);

    let started_at: u128 = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock to be after the Unix epoch")
      .as_nanos();

    let root: PathBuf = workspace_root.join("target").join("test-resources");

    collect_stale_generated_resources(
      &root,
      started_at.saturating_sub(GENERATED_RESOURCE_RETENTION.as_nanos()),
    );

    root.join(format!("{target}-{}-{started_at}", std::process::id()))
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

  /// A scratch tree inside this run's own directory, so a concurrent run never sees it.
  fn collector_root(case: &str) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("utils/collector/{case}"));

    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("scratch root");

    root
  }

  fn run_directory(root: &Path, name: &str) -> PathBuf {
    let path: PathBuf = root.join(name);

    fs::create_dir_all(path.join("nested")).expect("run directory");
    fs::write(path.join("nested").join("written.bin"), b"x").expect("run output");

    path
  }

  #[test]
  fn collects_run_directories_started_before_the_cutoff() {
    let root: PathBuf = collector_root("collected");
    let finished: PathBuf = run_directory(&root, "xrf_db-1234-1000");
    let running: PathBuf = run_directory(&root, "xrf_db-5678-3000");

    collect_stale_generated_resources(&root, 2000);

    assert!(!finished.exists(), "a run old enough to be finished is collected");
    // The whole point of the cutoff: a directory a live run may still be writing into must survive.
    assert!(running.exists(), "a run that may still be writing is left alone");
  }

  #[test]
  fn leaves_anything_that_is_not_a_run_directory_alone() {
    let root: PathBuf = collector_root("untouched");

    fs::create_dir_all(root.join("notes")).expect("directory");
    fs::create_dir_all(root.join("xrf_db-nope")).expect("directory");
    // Ends in a number old enough to collect, but names no process - a trailing timestamp alone is not a run.
    fs::create_dir_all(root.join("notes-1000")).expect("directory");
    fs::write(root.join("xrf_db-1234-1000"), b"a file, not a run").expect("file");

    collect_stale_generated_resources(&root, u128::MAX);

    assert!(
      root.join("notes").exists(),
      "a name that is not run-shaped is not ours to delete"
    );
    assert!(
      root.join("xrf_db-nope").exists(),
      "a name missing the timestamp is not run-shaped"
    );
    assert!(
      root.join("notes-1000").exists(),
      "a trailing number alone does not make a name run-shaped"
    );
    assert!(
      root.join("xrf_db-1234-1000").is_file(),
      "run-shaped or not, a file is never removed"
    );
  }
}
