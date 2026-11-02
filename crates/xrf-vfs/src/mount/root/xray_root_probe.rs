use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mount::XrayMountPlan;
use crate::{FsgameFile, XrayLogicalPath, XrayRootKind};

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

impl XrayRootProbe {
  /// Logical directories that identify an X-Ray root together: `fsgame.ltx` defines `$game_meshes$` and
  /// `$game_textures$` relative to `$game_data$`, so a directory holding both is the root those aliases resolve against.
  const MESHES_DIRECTORY: &str = "meshes";
  const TEXTURES_DIRECTORY: &str = "textures";

  /// Well-known entries a probe reports finding, in the order it reports them: the identifying pair, and further
  /// directories that are evidence of a root without identifying one.
  const PROBED_ENTRIES: [&str; 5] = [
    "configs",
    Self::MESHES_DIRECTORY,
    Self::TEXTURES_DIRECTORY,
    "sounds",
    "spawns",
  ];

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
  ///
  /// The nearest ancestor holding both `meshes` and `textures`, tried first so a nested `textures` directory does not
  /// hide a real gamedata root; failing that, the parent of the nearest asset directory the file sits in.
  pub fn implied_root(asset: impl AsRef<Path>) -> Option<PathBuf> {
    let asset: &Path = asset.as_ref();

    Self::find_complete_root(asset).or_else(|| Self::find_partial_root(asset))
  }

  /// The engine identity of a loose file, read against the root [`Self::implied_root`] finds for it.
  pub fn implied_logical_path(asset: impl AsRef<Path>) -> Option<XrayLogicalPath> {
    let asset: &Path = asset.as_ref();
    let root: PathBuf = Self::implied_root(asset)?;
    let components: Vec<&str> = asset
      .strip_prefix(&root)
      .ok()?
      .components()
      .map(|component| component.as_os_str().to_str())
      .collect::<Option<Vec<&str>>>()?;

    XrayLogicalPath::new(&components.join("\\")).ok()
  }

  /// The nearest installation containing a physical asset path: an ancestor holding an fsgame file, whether or not its
  /// gamedata holds loose `meshes` and `textures`.
  pub(crate) fn implied_installation(asset: &Path) -> Option<PathBuf> {
    asset
      .ancestors()
      .skip(1)
      .find(|candidate| FsgameFile::find(candidate).is_some())
      .map(Path::to_path_buf)
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
    Self::PROBED_ENTRIES
      .iter()
      .filter(|entry| path.join(entry).is_dir())
      .map(|entry| String::from(*entry))
      .collect()
  }

  /// The nearest ancestor holding every directory a root is identified by.
  fn find_complete_root(asset: &Path) -> Option<PathBuf> {
    asset
      .ancestors()
      .skip(1)
      .find(|candidate| {
        candidate.join(Self::MESHES_DIRECTORY).is_dir() && candidate.join(Self::TEXTURES_DIRECTORY).is_dir()
      })
      .map(Path::to_path_buf)
  }

  /// The parent of the nearest asset directory the file lives under, for a tree holding only one of them.
  fn find_partial_root(asset: &Path) -> Option<PathBuf> {
    asset
      .ancestors()
      .skip(1)
      .find(|candidate| {
        candidate
          .file_name()
          .and_then(|name| name.to_str())
          .is_some_and(|name| {
            name.eq_ignore_ascii_case(Self::MESHES_DIRECTORY) || name.eq_ignore_ascii_case(Self::TEXTURES_DIRECTORY)
          })
      })
      .and_then(Path::parent)
      .map(Path::to_path_buf)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::GeneratedResources;

  use super::{XrayRootKind, XrayRootProbe};

  fn tree(name: &str, directories: &[&str], files: &[(&str, &str)]) -> PathBuf {
    let root: PathBuf = GeneratedResources::create_dir(&format!("xray_root_probe/{name}")).expect("probe root");

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

  #[test]
  fn implies_the_root_holding_meshes_and_textures() {
    let root: PathBuf = tree("implied_gamedata", &["meshes/actors", "textures/act"], &[]);
    let visual: PathBuf = root.join("meshes/actors/stalker.ogf");

    assert_eq!(XrayRootProbe::implied_root(&visual).as_deref(), Some(root.as_path()));
  }

  #[test]
  fn implies_no_root_where_no_ancestor_looks_like_one() {
    let root: PathBuf = tree("implied_loose", &["desktop"], &[]);
    let visual: PathBuf = root.join("desktop/wpn_m4_hud.ogf");

    assert_eq!(XrayRootProbe::implied_root(&visual), None);
  }

  #[test]
  fn implies_the_root_of_a_tree_holding_only_one_asset_directory() {
    // A modder shipping textures alone still has files with exactly one engine reference, and refusing to read one
    // because no meshes were shipped beside it serves nobody.
    let root: PathBuf = tree("implied_textures_only", &["textures/act"], &[]);

    assert_eq!(
      XrayRootProbe::implied_root(root.join("textures/act/act_arm_1.dds")).as_deref(),
      Some(root.as_path())
    );

    let meshes: PathBuf = tree("implied_meshes_only", &["meshes/dynamics"], &[]);

    assert_eq!(
      XrayRootProbe::implied_root(meshes.join("meshes/dynamics/wpn.ogf")).as_deref(),
      Some(meshes.as_path())
    );
  }

  #[test]
  fn implies_a_complete_root_over_the_asset_directory_the_file_sits_in() {
    // Both rules answer for this file. The complete one is the gamedata tree, which is what every other root in the
    // search order is relative to; the loose one would name a directory halfway down it.
    let root: PathBuf = tree("implied_complete_wins", &["meshes", "textures/act/textures"], &[]);
    let texture: PathBuf = root.join("textures/act/textures/act_arm_1.dds");

    assert_eq!(XrayRootProbe::implied_root(&texture).as_deref(), Some(root.as_path()));
  }

  #[test]
  fn implies_the_nearest_root_when_one_tree_nests_another() {
    // An addon unpacked inside a gamedata tree resolves against itself, not its host.
    let outer: PathBuf = tree(
      "implied_nested",
      &["meshes", "textures", "mods/addon/meshes", "mods/addon/textures"],
      &[],
    );
    let inner: PathBuf = outer.join("mods/addon");
    let visual: PathBuf = inner.join("meshes/wpn.ogf");

    assert_eq!(XrayRootProbe::implied_root(&visual).as_deref(), Some(inner.as_path()));
  }

  #[test]
  fn implies_a_root_by_its_directories_rather_than_a_meshes_component() {
    // The word appearing in the path is not evidence; holding both directories is.
    let root: PathBuf = tree("implied_misleading", &["meshes/meshes", "textures"], &[]);
    let visual: PathBuf = root.join("meshes/meshes/wpn.ogf");

    assert_eq!(XrayRootProbe::implied_root(&visual).as_deref(), Some(root.as_path()));
  }

  #[test]
  fn implies_no_root_for_a_path_with_no_parent() {
    assert_eq!(XrayRootProbe::implied_root(Path::new("wpn.ogf")), None);
  }
}
