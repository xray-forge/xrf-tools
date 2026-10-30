use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use xrf_test_utils::GeneratedResources;

use crate::cache::{AppCache, AppCachePolicy, AppCacheStore, AppCacheTidy};
use crate::directories::AppDirectories;

const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// Fresh caches in a folder of their own, and that folder.
fn caches(name: &str) -> (AppCache, PathBuf) {
  let root: PathBuf = GeneratedResources::path(&format!("app_cache/{name}"));

  let _ = fs::remove_dir_all(&root);

  (AppCache::new(root.clone()), root)
}

/// Writes `entry` of `bytes` bytes into `store`, last used `age` ago, answering its file.
fn keep(root: &Path, store: &AppCacheStore, entry: &str, bytes: usize, age: Duration) -> PathBuf {
  let path: PathBuf = root.join(store.get_name()).join(entry);

  store.write(entry, &vec![7; bytes]).expect("the entry is written");
  set_last_use(&path, age);

  path
}

fn set_last_use(path: &Path, age: Duration) {
  OpenOptions::new()
    .write(true)
    .open(path)
    .and_then(|file| file.set_modified(SystemTime::now() - age))
    .expect("the entry's last use is set");
}

fn last_use_age(path: &Path) -> Duration {
  fs::metadata(path)
    .and_then(|metadata| metadata.modified())
    .expect("the entry has a last use")
    .elapsed()
    .unwrap_or_default()
}

#[test]
fn removes_what_went_unused_past_the_age_and_keeps_the_rest() {
  let (cache, root) = caches("unused_past_the_age");
  let renderer: AppCacheStore = cache.get_store("renderer");
  let materials: AppCacheStore = cache.get_store("materials");

  let fresh: PathBuf = keep(&root, &renderer, "fresh", 10, DAY);
  let stale: PathBuf = keep(&root, &renderer, "stale", 20, DAY * 9);
  let other: PathBuf = keep(&root, &materials, "zaton.json", 30, DAY * 30);

  let tidy: AppCacheTidy = cache.tidy(AppCachePolicy::DEFAULT, true);

  assert_eq!(
    tidy,
    AppCacheTidy {
      is_done: true,
      removed_entries: 2,
      removed_bytes: 50,
      kept_entries: 1,
      kept_bytes: 10,
    }
  );
  assert!(fresh.exists());
  assert!(!stale.exists());
  assert!(!other.exists());
}

#[test]
fn removes_the_least_recently_used_until_the_rest_fit_the_limit() {
  let (cache, root) = caches("least_recently_used");
  let store: AppCacheStore = cache.get_store("renderer");

  keep(&root, &store, "oldest", 400, DAY * 3);
  keep(&root, &store, "older", 300, DAY * 2);

  let newest: PathBuf = keep(&root, &store, "newest", 300, DAY);
  let policy: AppCachePolicy = AppCachePolicy {
    unused_days: 7,
    size_limit: 550,
  };
  let tidy: AppCacheTidy = cache.tidy(policy, true);

  assert_eq!((tidy.removed_entries, tidy.removed_bytes), (2, 700));
  assert_eq!((tidy.kept_entries, tidy.kept_bytes), (1, 300));
  assert!(newest.exists());
}

/// However full the caches and however old the files beside them, a tidy removes nothing outside the cache folder: the
/// level viewer's screenshots and bursts, saved views, logs, preferences and exports stay.
#[test]
fn removes_nothing_outside_the_cache_folder() {
  let data: PathBuf = GeneratedResources::path("app_cache/outside/appdata");
  let output: PathBuf = GeneratedResources::path("app_cache/outside/target");

  let _ = fs::remove_dir_all(GeneratedResources::path("app_cache/outside"));

  // The cache folder within the data directory, as `AppCache::of` takes it from `AppDirectories::get_cache_root`.
  let cache: AppCache = AppCache::new(data.join(AppDirectories::CACHE_DIRECTORY));
  let kept: Vec<PathBuf> = [
    data.join("level-viewer/screenshots/zaton.png"),
    data.join("level-viewer/bursts/rain/frame-000.png"),
    data.join("level-viewer/views/dusk.json"),
    data.join("logs/xrf-app.log"),
    data.join("preferences.json"),
    output.join("exports/textures/wood.dds"),
  ]
  .into_iter()
  .collect();

  for path in &kept {
    fs::create_dir_all(path.parent().expect("a folder")).expect("its folder is made");
    fs::write(path, [1; 64]).expect("the file is written");
    set_last_use(path, DAY * 365);
  }

  let store: AppCacheStore = cache.get_store("renderer");
  let stale: PathBuf = keep(
    &data.join(AppDirectories::CACHE_DIRECTORY),
    &store,
    "pipelines",
    64,
    DAY * 365,
  );
  let policy: AppCachePolicy = AppCachePolicy {
    unused_days: 1,
    size_limit: 0,
  };

  assert_eq!(cache.tidy(policy, true).removed_entries, 1);
  assert!(!stale.exists());
  assert!(kept.iter().all(|path| path.exists()), "{kept:?}");
}

#[test]
fn tidies_once_a_day_unless_forced() {
  let (cache, root) = caches("once_a_day");
  let store: AppCacheStore = cache.get_store("renderer");

  assert!(cache.tidy(AppCachePolicy::DEFAULT, false).is_done);

  let stale: PathBuf = keep(&root, &store, "stale", 10, DAY * 30);

  assert!(!cache.tidy(AppCachePolicy::DEFAULT, false).is_done);
  assert!(stale.exists());
  assert_eq!(cache.tidy(AppCachePolicy::DEFAULT, true).removed_entries, 1);
}

#[test]
fn leaves_a_tidy_another_instance_holds() {
  let (cache, root) = caches("held_by_another");

  fs::create_dir_all(&root).expect("the caches' folder is made");

  let lock: File = OpenOptions::new()
    .create(true)
    .truncate(false)
    .write(true)
    .open(root.join(".lock"))
    .expect("the lock opens");

  lock.lock().expect("the other instance holds the caches");

  assert!(!cache.tidy(AppCachePolicy::DEFAULT, true).is_done);
}

#[test]
fn a_read_marks_a_stale_entry_used_and_leaves_a_fresh_one() {
  let (cache, root) = caches("read_marks_used");
  let store: AppCacheStore = cache.get_store("renderer");

  let stale: PathBuf = keep(&root, &store, "stale", 4, DAY * 5);
  let fresh: PathBuf = keep(&root, &store, "fresh", 4, Duration::from_secs(120));

  assert_eq!(store.read("stale"), Some(vec![7; 4]));
  assert_eq!(store.read("fresh"), Some(vec![7; 4]));
  assert_eq!(store.read("missing"), None);
  assert!(last_use_age(&stale) < Duration::from_secs(60));
  assert!(last_use_age(&fresh) >= Duration::from_secs(119));
}
