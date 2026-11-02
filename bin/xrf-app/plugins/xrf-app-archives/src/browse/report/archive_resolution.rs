use serde::Serialize;
use xrf_archive::ArchiveProject;
use xrf_vfs::{XraySearch, XraySourceKind, label_from_path};

use crate::browse::report::{ArchiveResolutionSource, ArchiveResolutionVolume, ArchiveUnreadSource};

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
  pub fn of_search(search: &XraySearch) -> Self {
    Self {
      sources: search
        .list_sources()
        .into_iter()
        .map(ArchiveResolutionSource::from)
        .collect(),
      unread: search
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

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::sync::Arc;

  use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject};
  use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots, XraySourceKind};

  use super::ArchiveResolution;
  use crate::tests::fixtures;

  fn volume(name: &str, entries: usize) -> ArchiveDescriptor {
    ArchiveDescriptor {
      created_at: None,
      entries,
      modified_at: None,
      output_root_path: PathBuf::from("gamedata/"),
      mount_root: String::from("gamedata"),
      path: PathBuf::from(format!("C:/game/db/{name}")),
      size_compressed: 0,
      size_real: 0,
    }
  }

  fn descriptor(name: &str, is_directory: bool) -> ArchiveFileDescriptor {
    let mut descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(0, Arc::from(name), 0, 0, 0);

    descriptor.is_directory = is_directory;

    descriptor
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
    let front: PathBuf = fixtures::tree("resolution/front", &[("configs/system.ltx", b"payload")]);
    let back: PathBuf = fixtures::tree(
      "resolution/back",
      &[
        ("configs/system.ltx", b"payload"),
        ("configs/only_back.ltx", b"payload"),
      ],
    );

    let resolution: ArchiveResolution = ArchiveResolution::of_search(&fixtures::search(&XrayRoots::new([
      XrayRoot::new(front.clone(), XrayMountMode::Directory),
      XrayRoot::new(back.clone(), XrayMountMode::Directory),
    ])));

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
