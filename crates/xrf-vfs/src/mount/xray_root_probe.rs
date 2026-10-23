use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::mount::XrayMountPlan;
use crate::mount::xray_root::{
  CONFIGS_DIRECTORY, MESHES_DIRECTORY, SOUNDS_DIRECTORY, SPAWNS_DIRECTORY, TEXTURES_DIRECTORY, find_implied_asset_root,
};
use crate::{FsgameFile, XrayLogicalPath};

/// Well-known entries a probe reports finding, in the order it reports them.
const PROBED_ENTRIES: [&str; 5] = [
  CONFIGS_DIRECTORY,
  MESHES_DIRECTORY,
  TEXTURES_DIRECTORY,
  SOUNDS_DIRECTORY,
  SPAWNS_DIRECTORY,
];

/// What a path turns out to be when planned, and why.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRootProbe {
  /// What the path is, as planning sees it.
  pub kind: XrayRootKind,
  /// Which of the well-known entries sit directly beneath the path.
  pub evidence: Vec<String>,
  /// How many sources the path plans into, or zero when it plans into none.
  pub mounts: usize,
}

/// The kind a probed path belongs to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayRootKind {
  /// The path declares an installation with an fsgame file.
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

impl XrayRootKind {
  /// What a path is, before anything is opened or judged by its contents.
  pub(crate) fn of(path: &Path) -> Self {
    if FsgameFile::find(path).is_some() {
      Self::Installation
    } else if XrayMountPlan::is_volume(path) || XrayMountPlan::holds_volumes(path) {
      Self::Volumes
    } else {
      Self::Root
    }
  }

  /// The plan this kind means for `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when a declared fsgame file cannot be read, decoded, or parsed.
  pub(crate) fn plan(self, path: &Path) -> XrfResult<XrayMountPlan> {
    match self {
      Self::Installation => XrayMountPlan::from_fsgame(path),
      Self::Volumes => XrayMountPlan::volumes(path),
      // A directory nothing identifies is still mounted as the root it was named as. The two kinds only a probe
      // reports mean the same thing here, because each still describes a path a caller may ask to mount.
      Self::Root | Self::Unrecognized | Self::Missing => XrayMountPlan::root(path),
    }
  }
}

impl XrayRootProbe {
  /// Describes what `path` is without opening anything.
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
    let kind: XrayRootKind = XrayRootKind::of(path);

    let Ok(plan) = kind.plan(path) else {
      return Self {
        kind: XrayRootKind::Unrecognized,
        evidence,
        mounts: 0,
      };
    };

    Self {
      // Planning alone cannot tell game data from any other readable directory, so what is beneath it decides: a tree
      // holding nothing an engine would load is the mistake this probe exists to catch.
      kind: if kind == XrayRootKind::Root && evidence.is_empty() {
        XrayRootKind::Unrecognized
      } else {
        kind
      },
      evidence,
      mounts: plan.len(),
    }
  }

  /// The X-Ray root a physical asset path sits under, if any: what a search plan centred on the asset searches first.
  pub fn implied_root(asset: impl AsRef<Path>) -> Option<PathBuf> {
    find_implied_asset_root(asset.as_ref())
  }

  /// The engine identity of a loose file, read against the root [`Self::implied_root`] finds for it.
  pub fn implied_logical_path(asset: impl AsRef<Path>) -> Option<XrayLogicalPath> {
    let asset: &Path = asset.as_ref();
    let root: PathBuf = find_implied_asset_root(asset)?;
    let components: Vec<&str> = asset
      .strip_prefix(&root)
      .ok()?
      .components()
      .map(|component| component.as_os_str().to_str())
      .collect::<Option<Vec<&str>>>()?;

    XrayLogicalPath::new(&components.join("\\")).ok()
  }

  /// Whether a directory holds archive volumes directly, which makes it an archive source.
  pub fn holds_volumes(path: impl AsRef<Path>) -> bool {
    XrayMountPlan::holds_volumes(path.as_ref())
  }

  /// Whether a file is an archive volume, asked of the crate that owns the format.
  pub fn is_volume(path: impl AsRef<Path>) -> bool {
    XrayMountPlan::is_volume(path.as_ref())
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
  fn reports_an_enhanced_edition_installation_by_its_game_named_fsgame() {
    // Nothing at the root of an Enhanced Edition looks like game data; only the fsgame file identifies it.
    let root: PathBuf = tree(
      "enhanced_edition",
      &["gamedata/textures", "resources"],
      &[(
        "fsgame_soc.ltx",
        "$arch_dir$ = false| false| $fs_root$\n$game_data$ = false| true| $fs_root$| gamedata\\\n",
      )],
    );

    let probe: XrayRootProbe = XrayRootProbe::describe(&root);

    assert_eq!(probe.kind, XrayRootKind::Installation);
    assert!(probe.evidence.is_empty());
    assert_eq!(probe.mounts, 1);
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

  /// A loose file is named the way the root that implies it would list it, whatever separators the host used.
  #[test]
  fn names_a_loose_file_by_its_logical_path_inside_the_implied_root() {
    let root: PathBuf = tree("implied_logical_path", &["meshes", "textures/ston"], &[]);
    let texture: PathBuf = root.join("textures").join("ston").join("Ston_Beton05.dds");

    assert_eq!(XrayRootProbe::implied_root(&texture), Some(root.clone()));
    assert_eq!(
      XrayRootProbe::implied_logical_path(&texture).map(|path| path.as_str().to_owned()),
      Some(String::from("textures\\ston\\ston_beton05.dds"))
    );

    // The root's own parent holds no meshes or textures, so a file beside the root is inside no root at all.
    let beside: PathBuf = root.parent().expect("scratch root has a parent").join("loose.dds");

    assert_eq!(
      XrayRootProbe::implied_logical_path(&beside),
      None,
      "a file beside the root is not inside it"
    );
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
