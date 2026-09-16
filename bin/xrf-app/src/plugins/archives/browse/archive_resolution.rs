use std::path::PathBuf;

use serde::Serialize;
use xrf_archive::{ArchiveDescriptor, ArchiveProject};
use xrf_vfs::{XrayAssetSource, XrayProbe, XraySearchedSource, XraySkippedMount, XraySourceKind, label_from_path};

/// One volume of a set, in the order a lookup reaches it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResolutionVolume {
  /// The volume file itself.
  pub path: PathBuf,
  /// Entries its name table holds, before the merge shadows one of them.
  pub entries: usize,
  /// Bytes its entries occupy as stored.
  pub size_compressed: u64,
  /// Bytes its entries occupy once unpacked.
  pub size_real: u64,
  /// Root the volume unpacks under, from `[header] entry_point` with its alias stripped.
  pub output_root_path: PathBuf,
}

impl From<&ArchiveDescriptor> for ArchiveResolutionVolume {
  fn from(volume: &ArchiveDescriptor) -> Self {
    Self {
      entries: volume.entries,
      output_root_path: volume.output_root_path.clone(),
      path: volume.path.clone(),
      size_compressed: volume.size_compressed,
      size_real: volume.size_real,
    }
  }
}

/// One source the open subject searches, and everything that says why it is searched where it is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResolutionSource {
  /// Where the source lives: a loose tree's root, or the directory holding a volume set.
  pub path: PathBuf,
  /// Short name for the source — the directory or volume-set name.
  pub label: String,
  /// Whether the source is a loose tree or a set of volumes.
  pub kind: XraySourceKind,
  /// How the mount plan described it: an `fsgame.ltx` alias such as `$game_data$`, or `root` or `volumes` for a path
  /// named directly. Absent for a source no plan named.
  pub origin: Option<String>,
  /// The root whose declaration reached this source, as the open named it.
  pub step: String,
  /// Logical base the source mounts at; empty for a complete root.
  pub base: String,
  /// Engine identities the source answers for, before anything searched ahead of it shadows one.
  pub entries: usize,
  /// Volumes merged behind this source, in the order a lookup reaches them. Empty for a loose tree.
  pub volumes: Vec<ArchiveResolutionVolume>,
}

/// A source the plan named that could not be opened, so the search reaches nothing it holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveUnreadSource {
  /// Where the source that failed to open lives.
  pub path: PathBuf,
  /// How the plan described it, such as an `fsgame.ltx` alias.
  pub origin: String,
  /// Why it could not be opened.
  pub reason: String,
}

/// How the open subject answers an engine path: every source it searches, in the order it searches them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResolution {
  /// Sources searched, highest priority first: the first one holding an engine path is the one that answers for it.
  pub sources: Vec<ArchiveResolutionSource>,
  /// Sources named but never opened, which are absent from the search rather than empty within it.
  pub unread: Vec<ArchiveUnreadSource>,
}

impl ArchiveResolution {
  /// How a mounted world resolves, read off the mounts the search actually runs over.
  pub fn of_probe(probe: &XrayProbe) -> Self {
    Self {
      sources: probe
        .list_sources()
        .into_iter()
        .map(ArchiveResolutionSource::from)
        .collect(),
      unread: probe
        .list_skipped_sources()
        .into_iter()
        .map(ArchiveUnreadSource::from)
        .collect(),
    }
  }

  /// How a volume set resolves: one source, whose volumes are searched in reverse merge order.
  pub fn of_volumes(project: &ArchiveProject) -> Self {
    Self {
      sources: vec![ArchiveResolutionSource {
        base: String::new(),
        entries: project.files.values().filter(|file| !file.is_directory).count(),
        kind: XraySourceKind::Archive,
        label: label_from_path(&project.root),
        origin: Some(String::from("volumes")),
        path: project.root.clone(),
        step: label_from_path(&project.root),
        volumes: project
          .archives
          .iter()
          .rev()
          .map(ArchiveResolutionVolume::from)
          .collect(),
      }],
      unread: Vec::new(),
    }
  }
}

impl From<XraySearchedSource<'_>> for ArchiveResolutionSource {
  fn from(source: XraySearchedSource<'_>) -> Self {
    let mounted: &dyn XrayAssetSource = source.mount.get_source();

    Self {
      base: source.mount.get_base().to_string(),
      entries: mounted.count_entries(),
      kind: mounted.get_kind(),
      label: mounted.get_label().to_string(),
      origin: source.mount.get_origin().map(str::to_string),
      path: mounted.get_root_path().to_path_buf(),
      step: source.step.to_string(),
      // Reversed for the same reason a set's own report is: the later volume of a merge is the one a lookup answers
      // out of, so the volume listed first is the one asked first.
      volumes: mounted.list_volumes().iter().rev().map(Into::into).collect(),
    }
  }
}

impl From<&XraySkippedMount> for ArchiveUnreadSource {
  fn from(skipped: &XraySkippedMount) -> Self {
    Self {
      origin: skipped.origin.clone(),
      path: skipped.path.clone(),
      reason: skipped.reason.clone(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::fs;
  use std::path::PathBuf;
  use std::sync::Arc;

  use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject, ArchiveReadPolicy};
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
  use xrf_vfs::{XrayMountMode, XrayProbeStep, XrayRoot, XrayRoots, XraySourceKind, XrayVfs};

  use super::ArchiveResolution;

  fn volume(name: &str, entries: usize) -> ArchiveDescriptor {
    ArchiveDescriptor {
      created_at: None,
      entries,
      modified_at: None,
      output_root_path: PathBuf::from("gamedata/"),
      path: PathBuf::from(format!("C:/game/db/{name}")),
      size_compressed: 0,
      size_real: 0,
    }
  }

  fn descriptor(name: &str, is_directory: bool) -> ArchiveFileDescriptor {
    ArchiveFileDescriptor {
      crc: 0,
      is_directory,
      name: Arc::from(name),
      offset: 0,
      size_compressed: 0,
      size_real: 0,
      volume: 0,
    }
  }

  /// Writes a tree and returns its root, since a mount is decided by what is actually on disk.
  fn tree(name: &str, files: &[&str]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("archives/resolution/{name}"));

    let _ = fs::remove_dir_all(&root);

    for file in files {
      let path: PathBuf = root.join(file);

      fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("test tree is creatable");
      fs::write(&path, b"payload").expect("test file is writable");
    }

    root
  }

  #[test]
  fn a_volume_set_is_searched_from_the_volume_that_wins_the_merge() {
    let files = [descriptor("configs\\system.ltx", false), descriptor("configs\\", true)];

    let resolution: ArchiveResolution = ArchiveResolution::of_volumes(&ArchiveProject {
      archives: vec![volume("base.db0", 5), volume("patch.db1", 2)],
      files: files
        .into_iter()
        .map(|descriptor| (Arc::clone(&descriptor.name), descriptor))
        .collect::<HashMap<Arc<str>, ArchiveFileDescriptor>>(),
      read_policy: ArchiveReadPolicy::default(),
      root: PathBuf::from("C:/game/db"),
      shadowed: Vec::new(),
      size_real: 0,
    });

    let source = &resolution.sources[0];

    assert_eq!(
      resolution.sources.len(),
      1,
      "a set is one source wherever it is mounted"
    );
    assert_eq!(source.kind, XraySourceKind::Archive);
    assert_eq!(source.label, "db");
    assert_eq!(
      source.entries, 1,
      "the directory record is not an entry a lookup answers"
    );

    // The merge lets the later volume win, so the patch is the one a read of a shared name comes out of.
    assert_eq!(
      source
        .volumes
        .iter()
        .map(|volume| volume.path.to_string_lossy().into_owned())
        .collect::<Vec<String>>(),
      vec![
        String::from("C:/game/db/patch.db1"),
        String::from("C:/game/db/base.db0")
      ]
    );
  }

  #[test]
  fn a_world_is_searched_in_the_order_its_roots_were_declared() {
    let front: PathBuf = tree("front", &["configs/system.ltx"]);
    let back: PathBuf = tree("back", &["configs/system.ltx", "configs/only_back.ltx"]);

    let roots: XrayRoots = XrayRoots::new([
      XrayRoot::new(front.clone(), XrayMountMode::Directory),
      XrayRoot::new(back.clone(), XrayMountMode::Directory),
    ]);

    let mut vfs: XrayVfs = XrayVfs::new();
    let steps: Vec<XrayProbeStep> = roots
      .to_probe_plan()
      .expect("the roots plan")
      .mount_into(&mut vfs)
      .expect("the roots mount");

    let resolution: ArchiveResolution = ArchiveResolution::of_probe(&vfs.probe().with_steps(steps));

    assert_eq!(
      resolution
        .sources
        .iter()
        .map(|source| (source.path.as_path(), source.entries, source.origin.as_deref()))
        .collect::<Vec<(&std::path::Path, usize, Option<&str>)>>(),
      vec![(front.as_path(), 1, Some("root")), (back.as_path(), 2, Some("root"))],
      "the tree declared first is asked first, and each says what it holds before the one above it hides anything"
    );

    assert!(resolution.unread.is_empty(), "both trees opened");
  }
}
