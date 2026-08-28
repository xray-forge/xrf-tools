use std::path::{Path, PathBuf};

/// Logical directories an X-Ray root holds, and which identify it as one.
///
/// `fsgame.ltx` defines `$game_meshes$` and `$game_textures$` relative to `$game_data$`, so a directory holding both is
/// the root those aliases resolve against.
pub(crate) const MESHES_DIRECTORY: &str = "meshes";
pub(crate) const TEXTURES_DIRECTORY: &str = "textures";
/// Further directories a root may hold, which are evidence of one without identifying it.
pub(crate) const CONFIGS_DIRECTORY: &str = "configs";
pub(crate) const SOUNDS_DIRECTORY: &str = "sounds";
pub(crate) const SPAWNS_DIRECTORY: &str = "spawns";

/// The X-Ray root a physical asset path sits under, if any.
///
/// Walks upward from the asset and answers with the nearest ancestor holding both a `meshes` and a `textures` directory.
/// Nearest rather than furthest, so a gamedata tree nested inside another resolves against the one that contains the
/// asset.
///
/// Finding a root does not promise a reference resolves inside it: a source tree holds both directories while storing
/// textures one directory per texture under names that do not match their reference. Callers that need a resolvable root
/// must therefore fall through on a failed lookup rather than on a failed derivation.
///
/// Returns the nearest implied root, or `None` when no ancestor looks like one. Behind [`crate::XrayMountPlan::implied_root`].
pub(crate) fn find_implied_asset_root(path: &Path) -> Option<PathBuf> {
  path
    .ancestors()
    .skip(1)
    .find(|candidate| candidate.join(MESHES_DIRECTORY).is_dir() && candidate.join(TEXTURES_DIRECTORY).is_dir())
    .map(Path::to_path_buf)
}

/// Finds the nearest installation containing a physical asset path.
///
/// An installation is an ancestor containing `fsgame.ltx`. Unlike [`find_implied_asset_root`], this also finds installations
/// whose `gamedata/` has no loose `meshes/` and `textures/` directories. Returns `None` when no ancestor declares one.
pub(crate) fn implied_install_root(path: &Path) -> Option<PathBuf> {
  path
    .ancestors()
    .skip(1)
    .find(|candidate| candidate.join(crate::FsgameFile::FILE_NAME).is_file())
    .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::find_implied_asset_root;

  /// Builds a throwaway tree, since the answer is a filesystem fact rather than a string transformation.
  ///
  /// Written to the generated scratch root, never to a committed fixture directory, and scoped by name because tests in
  /// one binary share that root and run in parallel.
  fn tree(name: &str, directories: &[&str]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_root/{name}"));

    let _ = fs::remove_dir_all(&root);

    for directory in directories {
      fs::create_dir_all(root.join(directory)).expect("test tree is creatable");
    }

    root
  }

  #[test]
  fn finds_the_root_holding_meshes_and_textures() {
    let root: PathBuf = tree("gamedata", &["meshes/actors", "textures/act"]);
    let visual: PathBuf = root.join("meshes/actors/stalker.ogf");

    assert_eq!(find_implied_asset_root(&visual).as_deref(), Some(root.as_path()));
  }

  #[test]
  fn answers_none_when_no_ancestor_looks_like_a_root() {
    let root: PathBuf = tree("loose", &["desktop"]);
    let visual: PathBuf = root.join("desktop/wpn_m4_hud.ogf");

    assert_eq!(find_implied_asset_root(&visual), None);
  }

  #[test]
  fn requires_both_directories_rather_than_either() {
    let root: PathBuf = tree("meshes_only", &["meshes/dynamics"]);

    assert_eq!(find_implied_asset_root(&root.join("meshes/dynamics/wpn.ogf")), None);
  }

  #[test]
  fn prefers_the_nearest_root_when_one_tree_nests_another() {
    // An addon unpacked inside a gamedata tree resolves against itself, not its host.
    let outer: PathBuf = tree(
      "nested",
      &["meshes", "textures", "mods/addon/meshes", "mods/addon/textures"],
    );
    let inner: PathBuf = outer.join("mods/addon");
    let visual: PathBuf = inner.join("meshes/wpn.ogf");

    assert_eq!(find_implied_asset_root(&visual).as_deref(), Some(inner.as_path()));
  }

  #[test]
  fn ignores_a_meshes_component_that_is_not_a_root() {
    // The word appearing in the path is not evidence; holding both directories is.
    let root: PathBuf = tree("misleading", &["meshes/meshes", "textures"]);
    let visual: PathBuf = root.join("meshes/meshes/wpn.ogf");

    assert_eq!(find_implied_asset_root(&visual).as_deref(), Some(root.as_path()));
  }

  #[test]
  fn answers_none_for_a_path_with_no_parent() {
    assert_eq!(find_implied_asset_root(Path::new("wpn.ogf")), None);
  }
}
