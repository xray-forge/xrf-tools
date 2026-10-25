use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fileslice::FileSlice;

/// The files a test generates, kept in one directory per test process under the workspace's `target/test-resources`.
///
/// A resource is named relative to that directory; `sample_*` names group a test source's resources under its file
/// stem, so `file!()` and `x.chunk` give `<stem>/x.chunk`.
pub struct GeneratedResources;

impl GeneratedResources {
  /// How long a run's directory survives before a later run collects it.
  const RETENTION: Duration = Duration::from_secs(60 * 60);

  /// The absolute path of a generated resource in this run's directory; creates nothing.
  pub fn path(resource: &str) -> PathBuf {
    Self::root().join(resource)
  }

  /// A resource name grouped under the test source's file stem, joined with the host's separator.
  pub fn sample_name(file: &str, resource: &str) -> String {
    Path::new(&Self::sample_directory(file))
      .join(resource)
      .to_string_lossy()
      .into_owned()
  }

  /// The absolute path of a resource grouped under the test source's file stem.
  pub fn sample_path(file: &str, resource: &str) -> PathBuf {
    Self::path(&Self::sample_name(file, resource))
  }

  /// The absolute path of the directory a test source's resources are grouped in; creates nothing.
  pub fn sample_root(file: &str) -> PathBuf {
    Self::path(&Self::sample_directory(file))
  }

  /// Make an empty directory for a generated resource, clearing whatever an earlier run of the same test left there,
  /// and answer its absolute path.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the old directory cannot be removed or the new one made.
  pub fn create_dir(resource: &str) -> IoResult<PathBuf> {
    let directory: PathBuf = Self::path(resource);

    match std::fs::remove_dir_all(&directory) {
      Ok(()) => {}
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => return Err(error),
    }

    std::fs::create_dir_all(&directory)?;

    Ok(directory)
  }

  /// Write a generated resource and return where it landed.
  pub fn write<C: AsRef<[u8]>>(resource: &str, contents: C) -> IoResult<PathBuf> {
    let mut file: File = Self::overwrite(resource)?;

    file.write_all(contents.as_ref())?;
    drop(file);

    Ok(Self::path(resource))
  }

  /// Creates or truncates a generated resource, with the directories above it, and opens it to read and write.
  pub fn overwrite(resource: &str) -> IoResult<File> {
    Self::overwrite_path(Self::path(resource))
  }

  /// Creates or truncates the file at an already resolved `path`, with the directories above it, and opens it to read
  /// and write.
  ///
  /// # Panics
  ///
  /// Panics when `path` has no parent, which no file below a test root lacks.
  pub fn overwrite_path<P: AsRef<Path>>(path: P) -> IoResult<File> {
    let path: &Path = path.as_ref();

    std::fs::create_dir_all(path.parent().expect("a test file has a parent directory"))?;

    OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(true)
      .read(true)
      .open(path)
      .map_err(|error| Self::name_open_error(path, error))
  }

  /// Opens a generated resource for reading.
  pub fn open(resource: &str) -> IoResult<File> {
    let path: PathBuf = Self::path(resource);

    File::open(&path).map_err(|error| Self::name_open_error(&path, error))
  }

  /// Opens a generated resource as a slice.
  pub fn open_slice(resource: &str) -> IoResult<FileSlice> {
    Self::open(resource).map(FileSlice::new)
  }

  /// The relative directory a test source's resources are grouped in: its file stem.
  ///
  /// # Panics
  ///
  /// Panics when `file` names no file, which `file!()` always does.
  fn sample_directory(file: &str) -> String {
    Path::new(file)
      .file_stem()
      .expect("a test source has a file name")
      .to_string_lossy()
      .into_owned()
  }

  /// This run's directory, chosen on first use, which also collects the directories of runs long finished.
  fn root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();

    ROOT.get_or_init(|| {
      let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
      let workspace_root: &Path = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("test utility crate to be inside the workspace crates directory");
      let executable: String = std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
        .unwrap_or_else(|| String::from("test"));

      // Cargo names a test binary `<target>-<16 hex>`, and that hash moves whenever the toolchain or a dependency
      // does. Dropping it keeps a directory readable and groups every run of one target under the same prefix.
      let target: &str = executable
        .rsplit_once('-')
        .filter(|(_, hash)| hash.len() == 16 && hash.chars().all(|char| char.is_ascii_hexdigit()))
        .map_or(executable.as_str(), |(name, _)| name);

      let started_at: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock to be after the Unix epoch")
        .as_nanos();

      let root: PathBuf = workspace_root.join("target").join("test-resources");

      Self::collect_stale(&root, started_at.saturating_sub(Self::RETENTION.as_nanos()));

      root.join(format!("{target}-{}-{started_at}", std::process::id()))
    })
  }

  /// Remove the run directories under `root` that were started before `collect_before`.
  ///
  /// Only entries shaped `<target>-<pid>-<nanos>` are considered, so anything else left in the tree by hand is not this
  /// function's to delete. Best effort throughout - a directory another user owns, or one held open on Windows, is left
  /// where it is rather than failing a test run over housekeeping.
  fn collect_stale(root: &Path, collect_before: u128) {
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

  /// Names `path` in an open that failed, keeping the kind and the system's reason.
  fn name_open_error(path: &Path, error: IoError) -> IoError {
    IoError::new(
      error.kind(),
      format!("Failed to open test asset {}: {error}", path.display()),
    )
  }
}

#[cfg(test)]
mod tests {
  use std::fs::{self, File};
  use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
  use std::path::{Path, PathBuf};

  use crate::GeneratedResources;

  #[test]
  fn writes_a_resource_and_reports_where_it_landed() -> IoResult<()> {
    let path: PathBuf = GeneratedResources::write("generated_resources/written.txt", "contents")?;

    assert_eq!(fs::read(&path)?, b"contents");
    assert_eq!(path, GeneratedResources::path("generated_resources/written.txt"));

    Ok(())
  }

  #[test]
  fn writes_bytes_as_readily_as_text() -> IoResult<()> {
    let path: PathBuf = GeneratedResources::write("generated_resources/written.bin", [0xEF, 0xBB, 0xBF])?;

    assert_eq!(fs::read(path)?, vec![0xEF, 0xBB, 0xBF]);

    Ok(())
  }

  #[test]
  fn replaces_whatever_was_there_before() -> IoResult<()> {
    GeneratedResources::write("generated_resources/replaced.txt", "first")?;

    let path: PathBuf = GeneratedResources::write("generated_resources/replaced.txt", "second")?;

    // Truncating rather than appending, so a rerun does not read the previous run's tail.
    assert_eq!(fs::read(path)?, b"second");

    Ok(())
  }

  #[test]
  fn creates_the_directories_above_the_resource() -> IoResult<()> {
    let path: PathBuf = GeneratedResources::write("generated_resources/nested/deeper/file.txt", "x")?;

    assert!(path.is_file());

    Ok(())
  }

  #[test]
  fn generated_resources_stay_under_target() {
    let path: PathBuf = GeneratedResources::path("generated_resources/generated.bin");
    let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root: &Path = manifest_dir
      .parent()
      .and_then(Path::parent)
      .expect("test utility crate to be inside the workspace crates directory");

    assert!(path.starts_with(workspace_root.join("target").join("test-resources")));
  }

  #[test]
  fn groups_samples_under_the_source_file_stem() {
    let name: String = GeneratedResources::sample_name(file!(), "sample.bin");

    assert_eq!(
      name,
      Path::new("generated_resources").join("sample.bin").to_string_lossy()
    );
    assert_eq!(
      GeneratedResources::sample_path(file!(), "sample.bin"),
      GeneratedResources::path(&name)
    );
    assert_eq!(
      GeneratedResources::sample_root(file!()),
      GeneratedResources::path("generated_resources")
    );
  }

  #[test]
  fn clears_an_earlier_directory_before_answering_it() -> IoResult<()> {
    let leftover: PathBuf = GeneratedResources::write("generated_resources/cleared/leftover.txt", "x")?;
    let directory: PathBuf = GeneratedResources::create_dir("generated_resources/cleared")?;

    assert!(directory.is_dir());
    assert!(!leftover.exists(), "an earlier run's files are cleared");

    Ok(())
  }

  #[test]
  fn reads_back_what_the_overwrite_handle_wrote() -> IoResult<()> {
    let resource: &str = "generated_resources/handle.bin";
    let mut generated_file: File = GeneratedResources::overwrite(resource)?;

    generated_file.write_all(b"generated")?;
    drop(generated_file);

    assert_eq!(fs::read(GeneratedResources::path(resource))?, b"generated");
    assert_eq!(
      GeneratedResources::open_slice(resource)?.bytes_remaining(),
      b"generated".len()
    );
    assert_eq!(
      GeneratedResources::open(resource)?.metadata()?.len(),
      b"generated".len() as u64
    );

    Ok(())
  }

  #[test]
  fn names_the_path_of_a_failed_open() {
    let error: IoError = GeneratedResources::open("generated_resources/missing.bin").expect_err("a missing file");

    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert!(error.to_string().contains("missing.bin"), "Unexpected error: {error}");
  }

  /// A scratch tree inside this run's own directory, so a concurrent run never sees it.
  fn collector_root(case: &str) -> PathBuf {
    let root: PathBuf = GeneratedResources::path(&format!("generated_resources/collector/{case}"));

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

    GeneratedResources::collect_stale(&root, 2000);

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

    GeneratedResources::collect_stale(&root, u128::MAX);

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
