use std::path::PathBuf;

use crate::vfs::tests::fake_source::{FakeArchiveSource, directory};
use crate::{XrayAssetContainer, XrayLookupScope, XrayMountedEntry, XrayProbePlan, XrayProbeStep, XrayVfs};

/// A loose tree in front of a volume set, which is what an installation with unpacked files is.
fn layered(name: &str) -> XrayVfs {
  let loose: PathBuf = directory(name, &["configs/system.ltx", "configs/only_loose.ltx"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("directory mounts");
  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new(
        "configs",
        &["configs\\system.ltx", "configs\\only_archived.ltx"],
      )),
    )
    .expect("archive mounts");

  vfs
}

#[test]
fn a_winner_carries_the_copies_it_hides() {
  let vfs: XrayVfs = layered("mounted_entries_layered");

  let entries: Vec<XrayMountedEntry> = vfs.list_mounted_entries();

  assert_eq!(
    entries
      .iter()
      .map(XrayMountedEntry::get_logical_path)
      .collect::<Vec<&str>>(),
    vec![
      "configs\\only_archived.ltx",
      "configs\\only_loose.ltx",
      "configs\\system.ltx"
    ],
    "one entry per engine identity, ordered by it"
  );

  let shared: &XrayMountedEntry = &entries[2];

  assert!(
    matches!(shared.asset.get_container(), XrayAssetContainer::Directory { .. }),
    "the loose copy is the one a lookup reaches"
  );
  assert_eq!(shared.shadowed.len(), 1, "and it hides the archived copy");
  assert!(matches!(
    shared.shadowed[0].get_container(),
    XrayAssetContainer::Archive { .. }
  ));

  // The entries held once are exactly the ones nothing hides, which is what a badge in a browser reads.
  assert!(!entries[0].is_shadowing());
  assert!(!entries[1].is_shadowing());
  assert!(shared.is_shadowing());
}

#[test]
fn a_listing_carries_the_size_of_the_copy_that_wins() {
  let vfs: XrayVfs = layered("mounted_entries_size");

  let entries: Vec<XrayMountedEntry> = vfs.list_mounted_entries();
  let shared: &XrayMountedEntry = &entries[2];

  // The fixture writes its own name into every loose file, and the fake volume stores its label as each payload.
  assert_eq!(
    shared.size,
    "mounted_entries_size".len() as u64,
    "the winner is measured, not the copy behind it"
  );
  assert_eq!(
    vfs.read_size("configs\\system.ltx"),
    Some(shared.size),
    "and it agrees with asking for one size on its own"
  );
}

#[test]
fn a_scope_narrows_the_listing_the_way_it_narrows_a_lookup() {
  let loose: PathBuf = directory("mounted_entries_scope", &["configs/system.ltx", "textures/a.dds"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("directory mounts");

  let scope: XrayLookupScope = XrayLookupScope::all().with_prefix("configs").expect("prefix");

  assert_eq!(
    vfs
      .scoped(&scope)
      .list_mounted_entries()
      .iter()
      .map(XrayMountedEntry::get_logical_path)
      .collect::<Vec<&str>>(),
    vec!["configs\\system.ltx"]
  );
}

#[test]
fn a_probe_folds_a_later_step_behind_an_earlier_one() {
  // Two steps holding one path is the probe's own shadowing: the asset's own tree answers before the installation
  // behind it, and reporting only the winner would hide that arrangement exactly as dropping a mount would.
  let front: PathBuf = directory("mounted_entries_probe_front", &["configs/system.ltx"]);
  let back: PathBuf = directory(
    "mounted_entries_probe_back",
    &["configs/system.ltx", "configs/other.ltx"],
  );

  let mut vfs: XrayVfs = XrayVfs::new();

  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_root("front", &front)
    .expect("front plans")
    .with_root("back", &back)
    .expect("back plans")
    .mount_into(&mut vfs)
    .expect("both mount");

  let entries: Vec<XrayMountedEntry> = vfs.probe().with_steps(steps).list_mounted_entries();

  assert_eq!(
    entries
      .iter()
      .map(XrayMountedEntry::get_logical_path)
      .collect::<Vec<&str>>(),
    vec!["configs\\other.ltx", "configs\\system.ltx"]
  );

  assert_eq!(entries[1].asset.get_root(), Some(front.as_path()));
  assert_eq!(entries[1].shadowed.len(), 1);
  assert_eq!(entries[1].shadowed[0].get_root(), Some(back.as_path()));
}
