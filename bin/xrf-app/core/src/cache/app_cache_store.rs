use std::fs::{self, OpenOptions};
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use xrf_utils::StagedFile;

/// One client's cache in the application's caches: a folder of entries, each a file written whole or not at all. An
/// entry's modification time is when it was last used: set when it is written, and moved on when it is read, so the
/// caches' tidy (`AppCache::tidy`) removes what goes unused, as Bazel's disk cache and ccache do, with no index to keep
/// in step across processes.
#[derive(Clone, Debug)]
pub struct AppCacheStore {
  name: String,
  directory: PathBuf,
}

impl AppCacheStore {
  /// How stale an entry's last use may stand before a read moves it on, so a busy entry is not touched on every read.
  const USE_RESOLUTION: Duration = Duration::from_secs(60 * 60);

  pub(crate) fn new(name: &str, directory: PathBuf) -> Self {
    Self {
      name: name.to_owned(),
      directory,
    }
  }

  /// The client's name, its folder within the caches.
  pub fn get_name(&self) -> &str {
    &self.name
  }

  /// The entry `entry`'s bytes, marked used, or none where it is not kept or cannot be read.
  pub fn read(&self, entry: &str) -> Option<Vec<u8>> {
    let path: PathBuf = self.directory.join(entry);
    let bytes: Vec<u8> = fs::read(&path).ok()?;
    let last_used: Option<SystemTime> = fs::metadata(&path).and_then(|metadata| metadata.modified()).ok();

    log::info!(
      "Read the {} cache's '{entry}' ({} bytes), last used {}",
      self.name,
      bytes.len(),
      last_used
        .and_then(|time| time.elapsed().ok())
        .map_or_else(|| String::from("at an unknown time"), Self::format_age)
    );

    if last_used.is_none_or(|time| time.elapsed().is_ok_and(|age| age > Self::USE_RESOLUTION)) {
      Self::mark_used(&path);
    }

    Some(bytes)
  }

  /// Keeps `bytes` as the entry `entry`, replacing what was kept, its folder made where missing.
  ///
  /// # Errors
  ///
  /// Returns an IO error where the entry cannot be written whole.
  pub fn write(&self, entry: &str, bytes: &[u8]) -> IoResult<()> {
    StagedFile::write_creating_directories(&self.directory.join(entry), bytes)?;

    log::info!("Wrote the {} cache's '{entry}' ({} bytes)", self.name, bytes.len());

    Ok(())
  }

  /// Moves an entry's last use to now; one that cannot be touched keeps the last, and goes a little sooner.
  fn mark_used(path: &Path) {
    let marked: IoResult<()> = OpenOptions::new()
      .write(true)
      .open(path)
      .and_then(|file| file.set_modified(SystemTime::now()));

    if let Err(error) = marked {
      log::debug!("Could not mark '{}' used: {error}", path.display());
    }
  }

  fn format_age(age: Duration) -> String {
    let hours: u64 = age.as_secs() / 3600;

    match hours {
      0 => String::from("within the hour"),
      1..24 => format!("{hours} h ago"),
      _ => format!("{} days ago", hours / 24),
    }
  }
}
