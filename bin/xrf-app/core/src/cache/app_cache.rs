use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use xrf_utils::StagedFile;

use crate::cache::app_cache_policy::AppCachePolicy;
use crate::cache::app_cache_store::AppCacheStore;
use crate::cache::app_cache_tidy::AppCacheTidy;
use crate::directories::AppDirectories;

/// The application's caches, one folder for every client (the renderer's pipelines, and whatever else rebuilds from
/// nothing), tidied as one: entries unused past the policy's age go, then the least recently used until every cache
/// together fits its size limit. A tidy runs at most once a day unless forced, and only while no other instance of the
/// application holds the caches, as Cargo's cache cleanup skips a locked cache rather than wait.
#[derive(Clone, Debug)]
pub struct AppCache {
  root: PathBuf,
}

/// One entry found by a tidy: its file, its size and its last use.
struct AppCacheEntry {
  path: PathBuf,
  bytes: u64,
  last_used: SystemTime,
}

impl AppCache {
  /// The file one tidy at a time holds locked.
  const LOCK_FILE: &'static str = ".lock";
  /// The file whose modification time is when the caches were last tidied.
  const TIDIED_FILE: &'static str = ".tidied";
  /// How long after a tidy the next is due.
  const TIDY_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

  /// Whether a file at the caches' root named `name` is the caches' own bookkeeping, not anything a client keeps.
  pub fn is_own_file(name: &str) -> bool {
    name == Self::LOCK_FILE || name == Self::TIDIED_FILE
  }

  /// The caches in the application's cache folder (`AppDirectories::get_cache_root`), none where it has no data
  /// directory. They stand over that folder alone: a tidy removes files within it, and must never reach the tools'
  /// saved files or exports beside it.
  pub fn of(directories: &AppDirectories) -> Option<Self> {
    directories.get_cache_root().map(Self::new)
  }

  /// The caches kept in `root`, made when the first entry is written; the application's own are `AppCache::of`.
  pub(crate) fn new(root: PathBuf) -> Self {
    Self { root }
  }

  /// The cache of the client `name`, its folder within the caches.
  pub fn get_store(&self, name: &str) -> AppCacheStore {
    AppCacheStore::new(name, self.root.join(name))
  }

  /// Tidies the caches by `policy` where a tidy is due, or now where `is_forced`; a tidy another instance holds, or
  /// one that cannot take the lock, is left to it and says it was not done.
  pub fn tidy(&self, policy: AppCachePolicy, is_forced: bool) -> AppCacheTidy {
    if !is_forced && !self.is_tidy_due() {
      return AppCacheTidy::default();
    }

    let Some(lock) = self.try_lock() else {
      return AppCacheTidy::default();
    };
    let tidy: AppCacheTidy = Self::tidy_entries(self.list_entries(), policy, SystemTime::now());

    if let Err(error) = StagedFile::write(&self.root.join(Self::TIDIED_FILE), &[]) {
      log::warn!("Could not note when the caches were tidied: {error}");
    }

    drop(lock);

    log::info!(
      "Tidied the caches: removed {} entries ({} bytes), kept {} ({} bytes), by {} days unused and {} bytes at most",
      tidy.removed_entries,
      tidy.removed_bytes,
      tidy.kept_entries,
      tidy.kept_bytes,
      policy.unused_days,
      policy.size_limit
    );

    tidy
  }

  /// Whether no tidy ran within the interval, or none ever did.
  fn is_tidy_due(&self) -> bool {
    fs::metadata(self.root.join(Self::TIDIED_FILE))
      .and_then(|metadata| metadata.modified())
      .ok()
      .and_then(|tidied| tidied.elapsed().ok())
      .is_none_or(|age| age >= Self::TIDY_INTERVAL)
  }

  /// The caches' lock, held while the returned file lives; none where another holds it or it cannot be made.
  fn try_lock(&self) -> Option<File> {
    let file: IoResult<File> = fs::create_dir_all(&self.root).and_then(|()| {
      OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(self.root.join(Self::LOCK_FILE))
    });

    match file {
      Ok(file) => match file.try_lock() {
        Ok(()) => Some(file),
        Err(TryLockError::WouldBlock) => {
          log::info!("Another instance is tidying the caches; left to it");

          None
        }
        Err(TryLockError::Error(error)) => {
          log::warn!("Could not lock the caches to tidy them: {error}");

          None
        }
      },
      Err(error) => {
        log::warn!("Could not open the caches' lock: {error}");

        None
      }
    }
  }

  /// Removes from `entries` what went unused past the policy's age as of `now`, then the least recently used until the
  /// rest fit its size limit; an entry that cannot be removed counts as kept.
  fn tidy_entries(mut entries: Vec<AppCacheEntry>, policy: AppCachePolicy, now: SystemTime) -> AppCacheTidy {
    let unused: Duration = Duration::from_secs(u64::from(policy.unused_days) * 24 * 60 * 60);
    let mut tidy: AppCacheTidy = AppCacheTidy {
      is_done: true,
      ..AppCacheTidy::default()
    };

    // Oldest use first, so the size limit takes the least recently used.
    entries.sort_by_key(|entry| entry.last_used);

    let mut held: u64 = entries.iter().map(|entry| entry.bytes).sum();

    for entry in entries {
      let is_stale: bool = now.duration_since(entry.last_used).is_ok_and(|age| age > unused);

      if (is_stale || held > policy.size_limit) && Self::remove(&entry.path) {
        held -= entry.bytes;
        tidy.removed_entries += 1;
        tidy.removed_bytes += entry.bytes;
      } else {
        tidy.kept_entries += 1;
        tidy.kept_bytes += entry.bytes;
      }
    }

    tidy
  }

  /// Every entry of every client's cache, a file within a client's folder; the caches' own files at their root and the
  /// staged writes still on their way are not entries.
  fn list_entries(&self) -> Vec<AppCacheEntry> {
    let Ok(clients) = fs::read_dir(&self.root) else {
      return Vec::new();
    };
    let mut pending: Vec<PathBuf> = clients
      .flatten()
      .filter(|client| client.file_type().is_ok_and(|kind| kind.is_dir()))
      .map(|client| client.path())
      .collect();
    let mut entries: Vec<AppCacheEntry> = Vec::new();

    while let Some(directory) = pending.pop() {
      let Ok(children) = fs::read_dir(&directory) else {
        continue;
      };

      for child in children.flatten() {
        let Ok(metadata) = child.metadata() else {
          continue;
        };

        if metadata.is_dir() {
          pending.push(child.path());
        } else if !StagedFile::is_sibling(&child.file_name()) {
          entries.push(AppCacheEntry {
            path: child.path(),
            bytes: metadata.len(),
            last_used: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
          });
        }
      }
    }

    entries
  }

  fn remove(path: &Path) -> bool {
    match fs::remove_file(path) {
      Ok(()) => true,
      Err(error) => {
        log::debug!("Could not remove the cache entry '{}': {error}", path.display());

        false
      }
    }
  }
}
