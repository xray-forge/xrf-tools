use std::path::{Path, PathBuf};

use crate::vfs::tests::fixtures::{FakeArchiveSource, directory};
use crate::{XrayAsset, XrayAssetType, XrayLookupScope, XrayVfs};

#[test]
fn resolves_a_texture_reference_against_a_mounted_root() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory("", directory("texture", &["textures/wpn/wpn_ak74.dds"]))
    .expect("root mounts");

  let location: XrayAsset = vfs
    .resolve_all_in(&XrayLookupScope::all(), XrayAssetType::Dds, "wpn\\wpn_ak74")
    .expect("lookup succeeds")
    .pop()
    .expect("texture resolves");

  assert_eq!(location.get_logical_path().as_str(), "textures\\wpn\\wpn_ak74.dds");
}

#[test]
fn the_first_mount_holding_a_name_wins_and_the_shadowed_copy_stays_visible() {
  // Callers reverse engine declaration order so the first mount is the last-registered winner.
  let overlay: PathBuf = directory("overlay", &["textures/wpn/wpn_ak74.dds"]);
  let base: PathBuf = directory("base", &["textures/wpn/wpn_ak74.dds"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &overlay).expect("overlay mounts");
  vfs.mount_directory("", &base).expect("base mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();

  assert_eq!(
    vfs
      .find_in(&scope, "textures\\wpn\\wpn_ak74.dds")
      .unwrap()
      .and_then(|it| it.get_root().map(Path::to_path_buf)),
    Some(overlay)
  );
  assert_eq!(
    vfs.list_entries_all_in(&scope).len(),
    2,
    "the shadowed copy is still reportable"
  );
  assert_eq!(vfs.read_in(&scope, "textures\\wpn\\wpn_ak74.dds").unwrap(), b"overlay");
}

#[test]
fn a_subtree_mount_carries_engine_identity_through_its_base() {
  // A logical base lets a standalone subtree resolve against full engine paths.
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory("configs\\weapons", directory("subtree", &["ak74.ltx"]))
    .expect("subtree mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();

  assert!(vfs.find_in(&scope, "configs\\weapons\\ak74.ltx").unwrap().is_some());
  assert!(
    vfs.find_in(&scope, "ak74.ltx").unwrap().is_none(),
    "a source relative path is not a logical path"
  );
  assert_eq!(
    vfs
      .list_entries_in(&scope)
      .first()
      .map(|it| it.get_logical_path().to_string()),
    Some(String::from("configs\\weapons\\ak74.ltx"))
  );
}

#[test]
fn an_archived_entry_resolves_and_reads_but_offers_no_physical_path() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new("textures", &["textures\\wpn\\wpn_ak74.dds"])),
    )
    .expect("archive mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();
  let location: XrayAsset = vfs
    .find_in(&scope, "textures\\wpn\\wpn_ak74.dds")
    .unwrap()
    .expect("entry resolves");

  assert_eq!(location.to_physical_path(), None);
  assert_eq!(vfs.read_in(&scope, "textures\\wpn\\wpn_ak74.dds").unwrap(), b"textures");
}

#[test]
fn a_loose_file_overrides_an_archived_one() {
  // Reversing fsgame registration order puts loose gamedata ahead of archives.
  let loose: PathBuf = directory("loose_wins", &["textures/wpn/wpn_ak74.dds"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("directory mounts");
  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new("textures", &["textures\\wpn\\wpn_ak74.dds"])),
    )
    .expect("archive mounts");

  assert_eq!(
    vfs
      .read_in(&XrayLookupScope::all(), "textures\\wpn\\wpn_ak74.dds")
      .unwrap(),
    b"loose_wins"
  );
}

// Two steps over one mount, as a root named twice is, list its files once and hidden by nothing.
#[test]
fn a_mount_searched_by_two_steps_hides_none_of_its_own_files() {
  let loose: PathBuf = directory("repeated_step", &["configs/system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("directory mounts");

  let search: crate::XraySearch = crate::XraySearch::new(vfs)
    .with_step("first", XrayLookupScope::all())
    .with_step("second", XrayLookupScope::all());
  let entries = search.list_shadowing_entries();

  assert_eq!(entries.len(), 1);
  assert!(entries[0].shadowed.is_empty(), "a copy cannot hide itself");
}

// Handed the copy a loose file hides, a raw read and a parse both read that copy, never the winner.
#[test]
fn a_raw_read_and_a_parse_of_a_shadowed_copy_read_that_copy() {
  let loose: PathBuf = directory("shadowed_parse", &["configs/system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("directory mounts");
  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new("archived", &["configs\\system.ltx"])),
    )
    .expect("archive mounts");

  let search: crate::XraySearch = crate::XraySearch::new(vfs.with_cache_policy(crate::XrayCachePolicy::configs()))
    .with_step("all", XrayLookupScope::all());
  let shadowed: XrayAsset = search
    .list_shadowing_entries()
    .pop()
    .expect("the path is listed")
    .shadowed
    .pop()
    .expect("the archive's copy is hidden")
    .asset;
  let parse = |bytes: Vec<u8>| Ok(String::from_utf8(bytes).expect("text"));

  assert_eq!(search.read_asset(&shadowed).unwrap(), b"archived");
  assert_eq!(search.read_parsed(&shadowed, parse).unwrap().as_str(), "archived");

  let winner: XrayAsset = search.find("configs\\system.ltx").unwrap().expect("found");

  assert_eq!(search.read_parsed(&winner, parse).unwrap().as_str(), "shadowed_parse");
}

#[test]
fn a_writable_scope_skips_an_archive_entirely() {
  // A writable scope lets the same operation skip archives.
  let loose: PathBuf = directory("writable_scope", &["configs/system.ltx"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount("", Box::new(FakeArchiveSource::new("configs", &["configs\\other.ltx"])))
    .expect("archive mounts");
  vfs.mount_directory("", &loose).expect("directory mounts");

  let writable: XrayLookupScope = XrayLookupScope::writable();

  assert_eq!(
    vfs.list_entries_in(&writable).len(),
    1,
    "only the loose entry is in scope"
  );
  assert!(
    vfs.find_in(&writable, "configs\\other.ltx").unwrap().is_none(),
    "the archived entry is out of scope"
  );
}

#[test]
fn a_prefix_scope_cannot_answer_outside_its_subtree() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory(
      "",
      directory("prefixed", &["configs/system.ltx", "textures/wpn/wpn_ak74.dds"]),
    )
    .expect("root mounts");

  let configs: XrayLookupScope = XrayLookupScope::all().with_prefix("configs").expect("prefix is valid");

  assert!(vfs.find_in(&configs, "configs\\system.ltx").unwrap().is_some());
  assert!(
    vfs.find_in(&configs, "textures\\wpn\\wpn_ak74.dds").unwrap().is_none(),
    "a scoped lookup must not reach outside its subtree"
  );
  assert_eq!(vfs.list_entries_in(&configs).len(), 1);
}

#[test]
fn mounting_the_same_directory_twice_reuses_the_mount() {
  let root: PathBuf = directory("reused", &["configs/system.ltx"]);

  let mut vfs: XrayVfs = XrayVfs::new();
  let first = vfs.mount_directory("", &root).expect("mounts");
  let second = vfs.mount_directory("", &root).expect("mounts again");

  assert_eq!(first, second);
  assert_eq!(vfs.get_mounts().len(), 1, "the same root is not walked twice");
}

#[test]
fn mounting_one_directory_at_another_base_is_another_mount() {
  // Reusing the first mount would answer the second base's paths from the first base, so `weapons\ak74.ltx` would be
  // found nowhere the caller mounted it.
  let root: PathBuf = directory("rebased", &["ak74.ltx"]);

  let mut vfs: XrayVfs = XrayVfs::new();
  let root_mount = vfs.mount_directory("", &root).expect("mounts at the root");
  let based_mount = vfs
    .mount_directory("Configs\\Weapons", &root)
    .expect("mounts at a base");

  assert_ne!(root_mount, based_mount);
  assert_eq!(
    vfs.mount_directory("configs\\weapons", &root).expect("mounts again"),
    based_mount,
    "the same base, however spelled, reuses its mount"
  );
  assert!(
    vfs
      .find_in(&XrayLookupScope::all(), "configs\\weapons\\ak74.ltx")
      .expect("lookup")
      .is_some()
  );
}

#[test]
fn enumeration_dedupes_across_mounts_and_reports_shadowed_copies_separately() {
  let overlay: PathBuf = directory("dedupe_overlay", &["configs/system.ltx"]);
  let base: PathBuf = directory("dedupe_base", &["configs/system.ltx", "configs/weather.ltx"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &overlay).expect("overlay mounts");
  vfs.mount_directory("", &base).expect("base mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();

  assert_eq!(vfs.list_entries_in(&scope).len(), 2, "winners only");
  assert_eq!(vfs.list_entries_all_in(&scope).len(), 3, "including the shadowed copy");
}

#[test]
fn suffix_enumeration_matches_on_component_boundaries() {
  // `particles.xr` names that file anywhere in the tree; a neighbour that merely ends with the same characters is a
  // different asset and must not be counted as a second library.
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new(
        "suffixes",
        &["particles.xr", "mods\\particles.xr", "mods\\old_particles.xr"],
      )),
    )
    .expect("mounts");

  let found: Vec<String> = vfs
    .list_entries_in(&XrayLookupScope::all())
    .into_iter()
    .filter(|entry| {
      entry
        .get_logical_path()
        .ends_with("particles.xr")
        .expect("suffix is a valid fragment")
    })
    .map(|entry| entry.get_logical_path().to_string())
    .collect();

  assert_eq!(found, vec!["mods\\particles.xr", "particles.xr"]);
}
