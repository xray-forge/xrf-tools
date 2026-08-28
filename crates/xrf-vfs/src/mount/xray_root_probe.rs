use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::mount::xray_root::{
  CONFIGS_DIRECTORY, MESHES_DIRECTORY, SOUNDS_DIRECTORY, SPAWNS_DIRECTORY, TEXTURES_DIRECTORY,
};
use crate::{FsgameFile, XrayMountMode, XrayMountPlan};

/// Well-known entries a probe reports finding, in the order it reports them.
///
/// Evidence rather than a definition: holding `configs` does not make a directory a root, but saying which of these are
/// there is what lets a surface distinguish game data from a directory that merely mounts without error.
const PROBED_ENTRIES: [&str; 5] = [
  CONFIGS_DIRECTORY,
  MESHES_DIRECTORY,
  TEXTURES_DIRECTORY,
  SOUNDS_DIRECTORY,
  SPAWNS_DIRECTORY,
];

/// What a path turns out to be when planned, and why.
///
/// Exists because planning alone cannot answer the question a surface asks. [`XrayMountMode::Auto`] plans any readable
/// directory as a root, so a source repository and a game data tree plan identically; only the evidence separates them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRootProbe {
  /// What the path is, as planning sees it.
  pub kind: XrayRootKind,
  /// Which of the well-known entries sit directly beneath the path.
  ///
  /// Empty for an installation, whose content is behind its declaration rather than beside it, and empty for a
  /// directory holding nothing an engine would load.
  pub evidence: Vec<String>,
  /// How many sources the path plans into, or zero when it plans into none.
  pub mounts: usize,
}

/// The kind a probed path belongs to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayRootKind {
  /// The path declares an installation with `fsgame.ltx`.
  Installation,
  /// The path is one archive volume, or a directory of them.
  Volumes,
  /// The path is a directory holding content an engine would load.
  Root,
  /// The path is a directory, but nothing beneath it looks like game data.
  Unrecognized,
  /// Nothing is there, or it cannot be read.
  Missing,
}

impl XrayRootProbe {
  /// Describes what `path` is without opening anything.
  ///
  /// Planning is attempted, so an `fsgame.ltx` that is present but unreadable reports [`XrayRootKind::Unrecognized`]
  /// rather than an installation the caller cannot mount.
  pub fn describe(path: impl AsRef<Path>) -> Self {
    let path: &Path = path.as_ref();

    if !path.exists() {
      return Self {
        kind: XrayRootKind::Missing,
        evidence: Vec::new(),
        mounts: 0,
      };
    }

    let evidence: Vec<String> = Self::find_evidence(path);
    let plan: Option<XrayMountPlan> = XrayMountMode::Auto.plan(path).ok();
    let mounts: usize = plan.as_ref().map_or(0, XrayMountPlan::len);

    let kind: XrayRootKind = if plan.is_none() {
      XrayRootKind::Unrecognized
    } else if path.join(FsgameFile::FILE_NAME).is_file() {
      XrayRootKind::Installation
    } else if XrayMountPlan::is_volume(path) || XrayMountPlan::holds_volumes(path) {
      XrayRootKind::Volumes
    } else if evidence.is_empty() {
      XrayRootKind::Unrecognized
    } else {
      XrayRootKind::Root
    };

    Self { kind, evidence, mounts }
  }

  fn find_evidence(path: &Path) -> Vec<String> {
    PROBED_ENTRIES
      .iter()
      .filter(|entry| path.join(entry).is_dir())
      .map(|entry| String::from(*entry))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::{XrayRootKind, XrayRootProbe};

  fn tree(name: &str, directories: &[&str], files: &[(&str, &str)]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_root_probe/{name}"));

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("probe root");

    for directory in directories {
      fs::create_dir_all(root.join(directory)).expect("probe directory");
    }

    for (name, contents) in files {
      fs::write(root.join(name), contents).expect("probe file");
    }

    root
  }

  #[test]
  fn reports_a_loose_root_with_the_entries_it_found() {
    let root: PathBuf = tree("loose", &["configs", "meshes", "textures"], &[]);

    let probe: XrayRootProbe = XrayRootProbe::describe(&root);

    assert_eq!(probe.kind, XrayRootKind::Root);
    assert_eq!(probe.evidence, vec!["configs", "meshes", "textures"]);
    assert_eq!(probe.mounts, 1);
  }

  #[test]
  fn reports_an_installation_it_is_pointed_at() {
    let root: PathBuf = tree(
      "installation",
      &["gamedata/configs"],
      &[(
        "fsgame.ltx",
        "$fs_root$ = false| false| \n$game_data$ = false| true| $fs_root$| gamedata\\\n",
      )],
    );

    let probe: XrayRootProbe = XrayRootProbe::describe(&root);

    assert_eq!(probe.kind, XrayRootKind::Installation);
    assert!(probe.mounts > 0);
  }

  #[test]
  fn reports_a_directory_of_volumes() {
    let root: PathBuf = tree("volumes", &[], &[("gamedata.db0", ""), ("textures.db1", "")]);

    let probe: XrayRootProbe = XrayRootProbe::describe(&root);

    assert_eq!(probe.kind, XrayRootKind::Volumes);
    assert_eq!(probe.mounts, 1);
  }

  #[test]
  fn refuses_to_call_a_source_repository_game_data() {
    // The mistake this probe exists for: a repository plans as a root exactly like game data does, and only the
    // absence of anything an engine loads tells them apart.
    let root: PathBuf = tree("repository", &["src/engine/configs", "target/gamedata"], &[]);

    let probe: XrayRootProbe = XrayRootProbe::describe(&root);

    assert_eq!(probe.kind, XrayRootKind::Unrecognized);
    assert!(probe.evidence.is_empty());
  }

  #[test]
  fn reports_a_path_that_is_not_there() {
    let root: PathBuf = tree("missing", &[], &[]);

    let probe: XrayRootProbe = XrayRootProbe::describe(root.join("absent"));

    assert_eq!(probe.kind, XrayRootKind::Missing);
    assert_eq!(probe.mounts, 0);
    assert!(probe.evidence.is_empty());
  }
}
