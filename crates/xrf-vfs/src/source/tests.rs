use std::fs::{self, File, Metadata};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use xrf_test_utils::GeneratedResources;

use crate::source::XrayFileStamp;

/// A file of `contents` under the test's own folder, last modified `modified_ms` after the Unix epoch.
fn stamp_of(name: &str, contents: &str, modified_ms: u64) -> XrayFileStamp {
  let directory: PathBuf = GeneratedResources::create_dir("vfs/file_stamp").expect("the folder is made");
  let path: PathBuf = directory.join(name);

  fs::write(&path, contents).expect("the file is written");

  let modified: SystemTime = UNIX_EPOCH + Duration::from_millis(modified_ms);

  File::options()
    .write(true)
    .open(&path)
    .and_then(|file| file.set_modified(modified))
    .expect("the file's time is set");

  let metadata: Metadata = fs::metadata(&path).expect("the file is there");

  XrayFileStamp::of(&metadata)
}

/// The fingerprint folds the modification time in milliseconds, turned by 24 bits, over the size: what a render
/// source's texture stamp has always been, so a texture drawn before is not taken for a changed one.
#[test]
fn folds_the_time_and_the_size_into_a_fingerprint() {
  let modified_ms: u64 = 1_700_000_000_123;
  let stamp: XrayFileStamp = stamp_of("folded.dds", "texels", modified_ms);

  assert_eq!(stamp.get_fingerprint(), modified_ms.rotate_left(24) ^ 6);
}

#[test]
fn changes_the_fingerprint_with_the_size_or_the_time() {
  let stamp: XrayFileStamp = stamp_of("first.dds", "texels", 1_700_000_000_000);

  assert_ne!(
    stamp.get_fingerprint(),
    stamp_of("longer.dds", "more texels", 1_700_000_000_000).get_fingerprint()
  );
  assert_ne!(
    stamp.get_fingerprint(),
    stamp_of("later.dds", "texels", 1_700_000_001_000).get_fingerprint()
  );
}
