use serde::Serialize;
use xrf_archive::{ArchiveReadPolicy, ArchiveReadResult};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayAsset, XrayMountedEntry, XrayPathCollision, XrayProbe, XrayRoots};

use crate::plugins::archives::browse::archive_world_entry::ArchiveWorldEntry;

/// One mounted world the explorer browses: an installation, or any tree read as the engine would read it.
///
/// The other subject of the same explorer answers for one volume set and nothing else. Pointing that at a game folder
/// lists the archives and silently omits the loose `gamedata` tree in front of them, so it shows the archived payload
/// for files the engine would serve from disk. This one answers the other question: which copy actually wins, and what
/// that decision hides.
///
/// A listing rather than a VFS. The mounts live in the application's one [`crate::core::assets::AssetMountState`],
/// where every other surface's reads already go and where mounting one installation twice costs one index; what the
/// session owns is the answer it published.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveWorld {
  /// How the world was opened, so every later read of it addresses exactly these mounts.
  pub roots: XrayRoots,
  /// Sources searched, highest priority first, as each one names itself.
  pub mounts: Vec<String>,
  /// Winning entries, one per engine path, ordered by that path.
  pub files: Vec<ArchiveWorldEntry>,
  /// Entries a mount holds but no lookup can reach, folded during the open like the listing itself.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub collisions: Vec<XrayPathCollision>,
  pub read_policy: ArchiveReadPolicy,
  /// Unpacked bytes of the winning entries. What is shadowed is not counted; it is not what the engine would load.
  pub size_real: u64,
  /// Engine paths this world answers with more than one copy for.
  pub shadowed_count: usize,
}

impl ArchiveWorld {
  /// Lists everything a probe reaches, folded into the shape the explorer browses.
  ///
  /// Bounded by the installation rather than by any gesture, so callers run it off the executor.
  pub fn list(probe: &XrayProbe, roots: XrayRoots) -> Self {
    let entries: Vec<XrayMountedEntry> = probe.list_mounted_entries();

    Self {
      collisions: probe.list_collisions(),
      mounts: probe.list_roots(),
      read_policy: ArchiveReadPolicy::default(),
      roots,
      shadowed_count: entries.iter().filter(|entry| entry.is_shadowing()).count(),
      size_real: entries.iter().map(|entry| entry.size).sum(),
      files: entries.into_iter().map(ArchiveWorldEntry::from).collect(),
    }
  }

  /// The entry answering for one engine path, or `None` when this world holds none.
  ///
  /// A search rather than a scan, because [`Self::files`] is ordered by engine path and an installation puts tens of
  /// thousands of entries in it.
  pub fn find(&self, name: &str) -> Option<&ArchiveWorldEntry> {
    self
      .files
      .binary_search_by(|entry| entry.name.as_str().cmp(name))
      .ok()
      .map(|index| &self.files[index])
  }

  /// Reads one file of this world as text, subject to the viewer's read policy.
  ///
  /// The gate is asked against the published listing's size rather than the filesystem's, so the size that refuses a
  /// read is the size shown beside it.
  ///
  /// # Errors
  ///
  /// Returns an error when this world does not hold the path, the policy refuses it, or the bytes cannot be read or
  /// decoded.
  pub fn read_text(&self, probe: &XrayProbe, name: &str) -> XrfResult<ArchiveReadResult> {
    let entry: &ArchiveWorldEntry = self
      .find(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{name}' is not found in the opened game folder")))?;
    let size: u32 = entry.size_real.try_into().unwrap_or(u32::MAX);

    self.read_policy.require_text_read(name, size)?;

    let asset: XrayAsset = probe
      .find(name)?
      .get_asset()
      .cloned()
      .ok_or_else(|| XrfError::new_not_found_error(format!("'{name}' resolves to nothing in the mounted roots")))?;

    ArchiveReadResult::decode(name, &probe.read_asset_bytes(&asset)?, size)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
  use xrf_vfs::{XrayAssetContainer, XrayMountMode, XrayProbeStep, XrayRoot, XrayRoots, XrayVfs};

  use super::ArchiveWorld;

  fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("archives/world/{name}"));

    let _ = fs::remove_dir_all(&root);

    for (path, contents) in files {
      let path: PathBuf = root.join(path);

      fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("test tree is creatable");
      fs::write(&path, contents).expect("test file is writable");
    }

    root
  }

  /// A mod tree in front of the tree it overrides, which is the arrangement this subject exists to show.
  fn layered(name: &str) -> ArchiveWorld {
    let front: PathBuf = tree(
      &format!("{name}_front"),
      &[("configs/system.ltx", "front"), ("configs/only_front.ltx", "front")],
    );
    let back: PathBuf = tree(
      &format!("{name}_back"),
      &[("configs/system.ltx", "backish"), ("configs/only_back.ltx", "back")],
    );

    let roots: XrayRoots = XrayRoots::new([
      XrayRoot::new(front, XrayMountMode::Directory),
      XrayRoot::new(back, XrayMountMode::Directory),
    ]);

    let mut vfs: XrayVfs = XrayVfs::new();
    let steps: Vec<XrayProbeStep> = roots
      .to_probe_plan()
      .expect("the roots plan")
      .mount_into(&mut vfs)
      .expect("the roots mount");

    ArchiveWorld::list(&vfs.probe().with_steps(steps), roots)
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
    assert!(matches!(shared.shadowed[0], XrayAssetContainer::Directory { .. }));

    assert_eq!(world.shadowed_count, 1, "one path is answered twice");
    assert_eq!(world.mounts.len(), 2, "and both trees are named as searched");
  }

  #[test]
  fn a_world_counts_only_the_bytes_the_engine_would_load() {
    let world: ArchiveWorld = layered("size");

    // The shadowed copy is deliberately longer than the winner standing in front of it, so counting it would show.
    assert_eq!(
      world.size_real,
      ("front".len() + "front".len() + "back".len()) as u64,
      "the three winners, and not the copy one of them hides"
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
}
