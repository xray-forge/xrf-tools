//! What a snapshot keeps after parsing an asset, across mounts, forks and changes.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use xrf_error::XrfResult;

use crate::vfs::tests::fake_source::directory;
use crate::{XrayAssetType, XrayCachePolicy, XrayCacheStats, XrayLookupScope, XrayMountId, XrayRevision, XrayVfs};

const SYSTEM_LTX: &str = "configs\\system.ltx";

/// A snapshot retaining configs over one loose tree holding `system.ltx`, with the tree's mount id.
fn retaining(name: &str) -> (XrayVfs, XrayMountId) {
  let mut vfs: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::configs());
  let id: XrayMountId = vfs
    .mount_directory("", directory(name, &["configs/system.ltx"]))
    .expect("tree mounts");

  (vfs, id)
}

/// Parses `system.ltx` through `vfs`, counting the parses that actually ran.
fn parse(vfs: &XrayVfs, parses: &AtomicUsize) -> XrfResult<Arc<String>> {
  vfs.read_parsed_in(&XrayLookupScope::all(), XrayAssetType::Ltx, SYSTEM_LTX, |bytes| {
    parses.fetch_add(1, Ordering::Relaxed);

    Ok(String::from_utf8_lossy(&bytes).into_owned())
  })
}

#[test]
fn a_mount_keeps_what_is_retained() {
  let (mut vfs, _) = retaining("retention_mount");
  let parses: AtomicUsize = AtomicUsize::new(0);

  parse(&vfs, &parses).expect("parses");

  // A later mount wins nothing the earlier one holds, so the retained value still names the copy a lookup reaches.
  vfs
    .mount_directory("", directory("retention_mount_other", &["configs/game.ltx"]))
    .expect("the other tree mounts");
  parse(&vfs, &parses).expect("parses again");

  assert_eq!(parses.load(Ordering::Relaxed), 1, "the second read is served");
  assert_eq!(vfs.get_cache_stats().hits, 1);
}

#[test]
fn a_fork_shares_what_its_snapshot_retained() {
  let (vfs, _) = retaining("retention_fork");
  let parses: AtomicUsize = AtomicUsize::new(0);

  let first: Arc<String> = parse(&vfs, &parses).expect("parses");
  let fork: XrayVfs = vfs.fork();
  let second: Arc<String> = parse(&fork, &parses).expect("the fork parses");

  assert_eq!(parses.load(Ordering::Relaxed), 1);
  assert!(
    Arc::ptr_eq(&first, &second),
    "the fork is served the value its snapshot parsed"
  );

  let stats: XrayCacheStats = vfs.get_cache_stats();

  assert_eq!((stats.hits, stats.misses), (1, 1), "one account for the whole lineage");
  assert_eq!(fork.get_cache_stats(), stats);
}

#[test]
fn two_forks_mounting_other_trees_at_one_id_keep_their_values_apart() {
  let base: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::configs());
  let parses: AtomicUsize = AtomicUsize::new(0);
  let mut first: XrayVfs = base.fork();
  let mut second: XrayVfs = base.fork();

  let first_id: XrayMountId = first
    .mount_directory("", directory("retention_forks_first", &["configs/system.ltx"]))
    .expect("the first tree mounts");
  let second_id: XrayMountId = second
    .mount_directory("", directory("retention_forks_second", &["configs/system.ltx"]))
    .expect("the second tree mounts");

  assert_eq!(first_id, second_id, "both forks mount at the next free id");

  let first_value: Arc<String> = parse(&first, &parses).expect("the first fork parses");
  let second_value: Arc<String> = parse(&second, &parses).expect("the second fork parses");

  // The forks share one cache, and each copy is another file however equal their ids and paths.
  assert_eq!(parses.load(Ordering::Relaxed), 2);
  assert!(!Arc::ptr_eq(&first_value, &second_value));
}

#[test]
fn a_changed_revision_misses() {
  let (mut vfs, id) = retaining("retention_revision");
  let parses: AtomicUsize = AtomicUsize::new(0);

  parse(&vfs, &parses).expect("parses");
  vfs.set_file_revision(id, SYSTEM_LTX, XrayRevision::of(1));
  parse(&vfs, &parses).expect("parses the changed copy");

  assert_eq!(parses.load(Ordering::Relaxed), 2, "a copy that changed is parsed again");
}

#[test]
fn a_new_cache_policy_starts_a_lineage_of_its_own() {
  let (vfs, _) = retaining("retention_policy");
  let parses: AtomicUsize = AtomicUsize::new(0);

  parse(&vfs, &parses).expect("parses");

  let separate: XrayVfs = vfs.fork().with_cache_policy(XrayCachePolicy::configs());

  parse(&separate, &parses).expect("parses again");

  assert_eq!(parses.load(Ordering::Relaxed), 2);
  assert_eq!(vfs.get_cache_stats().hits, 0, "the original's account is its own");
}

#[test]
fn reports_an_absent_asset_as_not_found_without_parsing() {
  let (vfs, _) = retaining("retention_absent");
  let parses: AtomicUsize = AtomicUsize::new(0);

  let result = vfs.read_parsed_in(
    &XrayLookupScope::all(),
    XrayAssetType::Ltx,
    "configs\\absent.ltx",
    |bytes| {
      parses.fetch_add(1, Ordering::Relaxed);

      Ok(bytes.len())
    },
  );

  assert!(matches!(result, Err(xrf_error::XrfError::NotFound { .. })));
  assert_eq!(parses.load(Ordering::Relaxed), 0);
}
