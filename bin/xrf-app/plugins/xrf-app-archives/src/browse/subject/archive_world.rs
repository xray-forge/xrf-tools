use serde::Serialize;
use xrf_archive::{ArchiveReadPolicy, ArchiveReadResult};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayPathCollision, XraySearch, XrayShadowingEntry};

use crate::browse::subject::ArchiveWorldEntry;

/// One mounted world the explorer browses: an installation, or any tree read as the engine would read it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveWorld {
  /// The snapshot the listing was built from, which every read of this world goes through, so what is listed and
  /// what is read never come from two revisions.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub search: XraySearch,
  /// Sources searched, highest priority first, as each one names itself.
  pub mounts: Vec<String>,
  /// The same search order at the grain a container names — one entry per volume, not per merged mount.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub sources: Vec<String>,
  /// Winning entries, one per engine path, ordered by that path.
  pub files: Vec<ArchiveWorldEntry>,
  /// Entries a mount holds but no lookup can reach, folded during the open like the listing itself.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub collisions: Vec<XrayPathCollision>,
  /// Unpacked bytes of the winning entries. What is shadowed is not counted; it is not what the engine would load.
  pub size_real: u64,
  /// Engine paths this world answers with more than one copy for.
  pub shadowed_count: usize,
  /// Unpacked bytes held by the copies no lookup reaches.
  pub shadowed_size_real: u64,
}

impl ArchiveWorld {
  /// Lists everything `listed` reaches, folded into the shape the explorer browses, read later through `search`: the
  /// same snapshot, though `listed` may be a clone of it recording what the listing read.
  pub fn list(listed: &XraySearch, search: XraySearch) -> Self {
    let entries: Vec<XrayShadowingEntry> = listed.list_shadowing_entries();

    Self {
      collisions: listed.list_collisions(),
      mounts: listed.list_roots(),
      sources: listed.list_containers(),
      search,
      shadowed_count: entries.iter().filter(|entry| entry.is_shadowing()).count(),
      shadowed_size_real: entries.iter().map(XrayShadowingEntry::get_shadowed_size).sum(),
      size_real: entries.iter().map(XrayShadowingEntry::get_size).sum(),
      files: entries.into_iter().map(ArchiveWorldEntry::from).collect(),
    }
  }

  /// The entry answering for one engine path, however it is spelled, or `None` when this world holds none.
  pub fn find(&self, name: &str) -> Option<&ArchiveWorldEntry> {
    self.find_listed(name).or_else(|| {
      let identity: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;

      self.find_listed(identity.as_str())
    })
  }

  /// The entry listed under exactly `name`, which the listing holds normalized and in order.
  fn find_listed(&self, name: &str) -> Option<&ArchiveWorldEntry> {
    self
      .files
      .binary_search_by(|entry| entry.name.as_str().cmp(name))
      .ok()
      .map(|index| &self.files[index])
  }

  /// Reads one file of this world as text, if `policy` admits it.
  ///
  /// # Errors
  ///
  /// Returns an error when this world does not hold the path, the policy refuses it, or the bytes cannot be read or
  /// decoded.
  pub fn read_text(&self, name: &str, policy: &ArchiveReadPolicy) -> XrfResult<ArchiveReadResult> {
    let entry: &ArchiveWorldEntry = self
      .find(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{name}' is not found in the opened game folder")))?;
    let size: u32 = entry.size_real.try_into().unwrap_or(u32::MAX);

    policy.assert_can_read_as_text(name, size)?;

    ArchiveReadResult::decode(name, &self.search.read(name)?, size)
  }

  /// The mounted asset behind an engine path, in the snapshot this world was listed from.
  ///
  /// # Errors
  ///
  /// Returns an error when the path resolves to nothing there.
  pub fn find_asset(&self, name: &str) -> XrfResult<XrayAsset> {
    self
      .search
      .find(name)?
      .ok_or_else(|| XrfError::new_not_found_error(format!("'{name}' resolves to nothing in the mounted roots")))
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_archive::ARCHIVE_READ_POLICY;
  use xrf_vfs::{XrayAssetContainer, XrayMountMode, XrayRoot, XrayRoots};

  use super::ArchiveWorld;
  use crate::tests::fixtures;

  /// A mod tree in front of the tree it overrides, which is the arrangement this subject exists to show.
  fn layered(name: &str) -> ArchiveWorld {
    let front: PathBuf = fixtures::tree(
      &format!("world/{name}_front"),
      &[("configs/system.ltx", b"front"), ("configs/only_front.ltx", b"front")],
    );
    let back: PathBuf = fixtures::tree(
      &format!("world/{name}_back"),
      &[("configs/system.ltx", b"backish"), ("configs/only_back.ltx", b"back")],
    );

    fixtures::world(&XrayRoots::new([
      XrayRoot::new(front, XrayMountMode::Directory),
      XrayRoot::new(back, XrayMountMode::Directory),
    ]))
  }

  #[test]
  fn a_world_lists_one_entry_per_engine_path_with_what_it_hides() {
    let world: ArchiveWorld = layered("listing");

    assert_eq!(
      world
        .files
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<&str>>(),
      vec![
        "configs\\only_back.ltx",
        "configs\\only_front.ltx",
        "configs\\system.ltx"
      ]
    );

    let shared: &super::ArchiveWorldEntry = world.find("configs\\system.ltx").expect("the shared path is listed");

    assert_eq!(shared.size_real, "front".len() as u64, "the winner is what is measured");
    assert_eq!(shared.shadowed.len(), 1);
    assert!(matches!(
      shared.shadowed[0].container,
      XrayAssetContainer::Directory { .. }
    ));

    // The fixture gives the two copies different lengths, so a hidden copy reporting the winner's size would be
    // indistinguishable from one reporting its own.
    assert_eq!(
      shared.shadowed[0].size_real,
      "backish".len() as u64,
      "a hidden copy is measured by the mount holding it"
    );

    assert_eq!(world.shadowed_count, 1, "one path is answered twice");
    assert_eq!(world.mounts.len(), 2, "and both trees are named as searched");
  }

  #[test]
  fn a_world_counts_only_the_bytes_the_engine_would_load() {
    let world: ArchiveWorld = layered("size");

    assert_eq!(
      world.size_real,
      ("front".len() + "front".len() + "back".len()) as u64,
      "the three winners, and not the copy one of them hides"
    );

    assert_eq!(
      world.shadowed_size_real,
      "backish".len() as u64,
      "what the arrangement costs is counted apart from what the engine would load"
    );
  }

  #[test]
  fn a_world_finds_an_entry_by_the_order_it_published() {
    let world: ArchiveWorld = layered("find");

    for entry in &world.files {
      assert_eq!(world.find(&entry.name).map(|found| &found.name), Some(&entry.name));
    }

    assert!(world.find("configs\\absent.ltx").is_none());
  }

  #[test]
  fn a_world_finds_an_entry_however_a_reference_spells_it() {
    let world: ArchiveWorld = layered("spelling");

    assert_eq!(
      world.find("Configs/System.LTX").map(|found| found.name.as_str()),
      Some("configs\\system.ltx"),
      "a reference authored in another case or with forward slashes is the same engine path"
    );
  }

  #[test]
  fn a_world_reads_from_the_snapshot_it_was_listed_from() {
    let world: ArchiveWorld = layered("snapshot");

    assert_eq!(
      world
        .read_text("configs\\system.ltx", &ARCHIVE_READ_POLICY)
        .expect("the winner reads")
        .content,
      "front"
    );
  }
}
