use std::path::{Path, PathBuf};

use crate::vfs::tests::fake_source::{FakeArchiveSource, directory};
use crate::{XrayAsset, XrayLookupScope, XrayVfs};

#[test]
fn resolves_a_texture_reference_against_a_mounted_root() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory("", directory("texture", &["textures/wpn/wpn_ak74.dds"]))
    .expect("root mounts");

  let location: XrayAsset = vfs
    .resolve_dds_texture("wpn\\wpn_ak74")
    .expect("lookup succeeds")
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
      .scoped(&scope)
      .find("textures\\wpn\\wpn_ak74.dds")
      .unwrap()
      .and_then(|it| it.get_root().map(Path::to_path_buf)),
    Some(overlay)
  );
  assert_eq!(
    vfs
      .scoped(&scope)
      .find_all("textures\\wpn\\wpn_ak74.dds")
      .unwrap()
      .len(),
    2,
    "the shadowed copy is still reportable"
  );
  assert_eq!(
    vfs.scoped(&scope).read_bytes("textures\\wpn\\wpn_ak74.dds").unwrap(),
    b"overlay"
  );
}

#[test]
fn a_subtree_mount_carries_engine_identity_through_its_base() {
  // A logical base lets a standalone subtree resolve against full engine paths.
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory("configs\\weapons", directory("subtree", &["ak74.ltx"]))
    .expect("subtree mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();

  assert!(vfs.scoped(&scope).find("configs\\weapons\\ak74.ltx").unwrap().is_some());
  assert!(
    vfs.scoped(&scope).find("ak74.ltx").unwrap().is_none(),
    "a source relative path is not a logical path"
  );
  assert_eq!(
    vfs
      .scoped(&scope)
      .list_entries()
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
    .scoped(&scope)
    .find("textures\\wpn\\wpn_ak74.dds")
    .unwrap()
    .expect("entry resolves");

  assert_eq!(location.to_physical_path(), None);
  assert_eq!(
    vfs.scoped(&scope).read_bytes("textures\\wpn\\wpn_ak74.dds").unwrap(),
    b"textures"
  );
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

  assert_eq!(vfs.read_bytes("textures\\wpn\\wpn_ak74.dds").unwrap(), b"loose_wins");
}

#[test]
fn writing_an_archived_winner_is_refused_and_names_the_archive() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new("configs", &["configs\\system.ltx"])),
    )
    .expect("archive mounts");

  let error: String = vfs
    .write(&XrayLookupScope::all(), "configs\\system.ltx", b"formatted")
    .expect_err("write is refused")
    .to_string();

  assert!(error.contains("read only"), "{error}");
  assert!(error.contains("configs"), "the refusal names what holds it: {error}");
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

  vfs
    .write(&writable, "configs\\system.ltx", b"formatted")
    .expect("a loose winner is writable");

  assert_eq!(
    vfs.scoped(&writable).list_entries().len(),
    1,
    "only the loose entry is in scope"
  );
  assert!(
    vfs.scoped(&writable).find("configs\\other.ltx").unwrap().is_none(),
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

  assert!(vfs.scoped(&configs).find("configs\\system.ltx").unwrap().is_some());
  assert!(
    vfs
      .scoped(&configs)
      .find("textures\\wpn\\wpn_ak74.dds")
      .unwrap()
      .is_none(),
    "a scoped lookup must not reach outside its subtree"
  );
  assert_eq!(vfs.scoped(&configs).list_entries().len(), 1);
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
fn enumeration_dedupes_across_mounts_and_reports_shadowed_copies_separately() {
  let overlay: PathBuf = directory("dedupe_overlay", &["configs/system.ltx"]);
  let base: PathBuf = directory("dedupe_base", &["configs/system.ltx", "configs/weather.ltx"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &overlay).expect("overlay mounts");
  vfs.mount_directory("", &base).expect("base mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();

  assert_eq!(vfs.scoped(&scope).list_entries().len(), 2, "winners only");
  assert_eq!(
    vfs.scoped(&scope).list_entries_all().len(),
    3,
    "including the shadowed copy"
  );
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

  let mut found: Vec<String> = vfs
    .list_entries_with_suffix("particles.xr")
    .expect("suffix is a valid fragment")
    .into_iter()
    .map(|entry| entry.get_logical_path().to_string())
    .collect();

  found.sort();

  assert_eq!(found, vec!["mods\\particles.xr", "particles.xr"]);
}
