use std::fmt::{Display, Formatter, Result as FormatResult};
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::mount::xray_root::implied_install_root;
use crate::{FsgameFile, XrayMountPlan, XrayRootKind};

/// How a caller's path is turned into mounts.
///
/// One vocabulary for every tool, so `--source` means the same thing everywhere. Each variant maps onto an
/// [`XrayMountPlan`] constructor; this exists so a command surface, an app setting, and an editor can all name
/// the choice rather than each re-deriving it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayMountMode {
  /// Treat the path as an installation when it declares one, as one volume when it is one, as a volume set when it
  /// holds volumes, and as a complete root otherwise.
  #[default]
  Auto,
  /// Treat the path as a complete X-Ray root, ignoring any `fsgame.ltx` beside it.
  Directory,
  /// Treat the path as one archive volume, or as every volume beneath a directory, and mount each on its own.
  ///
  /// What the engine does for a path declared with `recurs = true`: `CLocatorAPI::ProcessOne` hands any `.db*` or
  /// `.xdb*` file it meets to `ProcessArchive`, including one `Recurse` found in a subdirectory
  /// (`xray-16/src/xrCore/LocatorAPI.cpp`). `Auto` is the `recurs = false` half of the same rule, which is how Anomaly
  /// declares `$arch_dir$` and each of its subdirectories.
  ///
  /// Name it for a path a person picked rather than one `fsgame.ltx` declared, where a listing already read that path
  /// recursively and every entry it lists must be readable back.
  Volumes,
  /// Require the path to declare an installation, and mount everything it declares.
  Installation,
  /// Mount the nearest installation containing the path, searching upwards for `fsgame.ltx`.
  ContainingInstallation,
}

impl XrayMountMode {
  /// Builds the plan this mode means for `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when [`Self::Installation`] is asked for a path that declares none, or when a declared
  /// `fsgame.ltx` cannot be read, decoded, or parsed.
  pub fn plan(&self, path: impl AsRef<Path>) -> XrfResult<XrayMountPlan> {
    let path: &Path = path.as_ref();

    match self {
      // Classified rather than branched on here, so that what a path is has one answer: a surface describing a path
      // and this planning it must never disagree. A directory of volumes is neither an installation nor a loose root,
      // and mounting it as the latter answers for `textures.db0` instead of for the assets inside it - which is what
      // the fsgame planner already avoids when it meets the same directory through a declaration.
      Self::Auto => XrayRootKind::of(path).plan(path),
      Self::Directory => XrayMountPlan::root(path),
      Self::Volumes => XrayMountPlan::nested_volumes(path),
      Self::Installation => XrayMountPlan::from_fsgame(path),
      Self::ContainingInstallation => {
        if Self::declares_installation(path) {
          return XrayMountPlan::from_fsgame(path);
        }

        match implied_install_root(path) {
          Some(install) => XrayMountPlan::from_fsgame(install),
          None => XrayMountPlan::root(path),
        }
      }
    }
  }

  /// Whether a path itself declares an installation.
  pub fn declares_installation(path: impl AsRef<Path>) -> bool {
    path.as_ref().join(FsgameFile::FILE_NAME).is_file()
  }
}

impl Display for XrayMountMode {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    formatter.write_str(match self {
      Self::Auto => "auto",
      Self::Directory => "directory",
      Self::Volumes => "volumes",
      Self::Installation => "installation",
      Self::ContainingInstallation => "containing-installation",
    })
  }
}

impl TryFrom<&str> for XrayMountMode {
  type Error = xrf_error::XrfError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "auto" => Ok(Self::Auto),
      "directory" => Ok(Self::Directory),
      "volumes" => Ok(Self::Volumes),
      "installation" => Ok(Self::Installation),
      "containing-installation" => Ok(Self::ContainingInstallation),
      other => Err(xrf_error::XrfError::new_asset_error(format!(
        "unknown source mode '{other}', expected auto, directory, volumes, installation or containing-installation"
      ))),
    }
  }
}

/// Delegates to [`TryFrom<&str>`], so a mode parses out of a command flag or a config value with `parse()`.
impl FromStr for XrayMountMode {
  type Err = xrf_error::XrfError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    Self::try_from(value)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::XraySourceKind;

  use super::XrayMountMode;

  fn install(name: &str) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_mount_mode/{name}"));

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(root.join("gamedata").join("configs")).expect("gamedata");
    fs::write(
      root.join("fsgame.ltx"),
      "$fs_root$ = false| false| \n$game_data$ = false| true| $fs_root$| gamedata\\\n",
    )
    .expect("fsgame");

    root
  }

  fn volumes(name: &str, names: &[&str]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_mount_mode/{name}"));

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("volumes root");

    for name in names {
      let volume: PathBuf = root.join(name);

      fs::create_dir_all(volume.parent().expect("volume parent")).expect("volume directory");
      fs::write(volume, []).expect("volume");
    }

    root
  }

  #[test]
  fn auto_reads_an_installation_it_is_pointed_at() {
    let root: PathBuf = install("auto_install");

    assert!(!XrayMountMode::Auto.plan(&root).expect("plan").is_empty());
    assert!(XrayMountMode::declares_installation(&root));
  }

  #[test]
  fn auto_does_not_search_upwards() {
    // Pointing at a subdirectory must not widen into the whole installation, or a command silently changes what it touches.
    let root: PathBuf = install("auto_subdirectory");
    let configs: PathBuf = root.join("gamedata").join("configs");

    let plan = XrayMountMode::Auto.plan(&configs).expect("plan");

    assert_eq!(plan.len(), 1, "only the named directory is planned");
    assert!(plan.get_mounts()[0].path.ends_with("configs"));
  }

  #[test]
  fn auto_mounts_a_named_volume_as_itself() {
    // Planning never reads the volume, so an empty file is enough to reach the branch - and reaching it is the point:
    // planned through its directory instead, the mount would also carry every sibling volume the caller did not name.
    let root: PathBuf = volumes("auto_volume", &["gamedata.db0", "textures.db1"]);
    let volume: PathBuf = root.join("gamedata.db0");

    let plan = XrayMountMode::Auto.plan(&volume).expect("plan");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get_mounts()[0].origin, "volumes");
    assert_eq!(plan.get_mounts()[0].path, volume);
  }

  #[test]
  fn auto_still_mounts_a_directory_of_volumes() {
    let root: PathBuf = volumes("auto_volume_set", &["gamedata.db0", "textures.db1"]);

    let plan = XrayMountMode::Auto.plan(&root).expect("plan");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get_mounts()[0].origin, "volumes");
    assert_eq!(plan.get_mounts()[0].path, root);
  }

  #[test]
  fn auto_leaves_a_volume_below_the_directory_unmounted() {
    // Non-recursive on purpose: `fsgame.ltx` declares each volume directory separately, so a declared alias means the
    // volumes directly under it. `Volumes` is the mode for a path a person named instead.
    let root: PathBuf = volumes("auto_nested_volume", &["gamedata.db0", "textures/textures.db1"]);

    let plan = XrayMountMode::Auto.plan(&root).expect("plan");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get_mounts()[0].path, root);
  }

  #[test]
  fn volumes_mounts_every_volume_beneath_the_directory() {
    // What an archive project lists is what this must reach, and it lists recursively.
    let root: PathBuf = volumes(
      "volumes_nested",
      &["gamedata.db0", "textures/textures.db1", "patches/xpatch.db"],
    );

    let plan = XrayMountMode::Volumes.plan(&root).expect("plan");
    let paths: Vec<PathBuf> = plan.get_mounts().iter().map(|mount| mount.path.clone()).collect();

    // Reverse of the order the project merges them in, which is the name order `Recurse` registers them in: no
    // directory is special, so the volume its name table names answers the lookup.
    assert_eq!(
      paths,
      vec![
        root.join("textures").join("textures.db1"),
        root.join("patches").join("xpatch.db"),
        root.join("gamedata.db0"),
      ]
    );
    assert!(
      plan
        .get_mounts()
        .iter()
        .all(|mount| mount.kind == XraySourceKind::Archive && mount.base.is_empty())
    );
  }

  #[test]
  fn volumes_mounts_a_named_volume_as_itself() {
    let root: PathBuf = volumes("volumes_named", &["gamedata.db0", "textures.db1"]);
    let volume: PathBuf = root.join("gamedata.db0");

    let plan = XrayMountMode::Volumes.plan(&volume).expect("plan");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get_mounts()[0].path, volume);
  }

  #[test]
  fn volumes_plans_nothing_where_there_are_none() {
    let root: PathBuf = volumes("volumes_absent", &["readme.txt"]);

    assert!(XrayMountMode::Volumes.plan(&root).expect("plan").is_empty());
  }

  #[test]
  fn containing_installation_searches_upwards() {
    let root: PathBuf = install("containing");
    let configs: PathBuf = root.join("gamedata").join("configs");

    let plan = XrayMountMode::ContainingInstallation.plan(&configs).expect("plan");

    assert!(
      plan
        .get_mounts()
        .iter()
        .any(|mount| mount.path.ends_with("gamedata") && mount.base.is_empty()),
      "the installation above the path is planned, not the path itself"
    );
  }

  #[test]
  fn containing_installation_prefers_a_declaration_at_the_path_itself() {
    let root: PathBuf = install("containing_self");

    let plan = XrayMountMode::ContainingInstallation.plan(&root).expect("plan");

    assert!(
      plan.get_mounts().iter().any(|mount| mount.path.ends_with("gamedata")),
      "a declaration here wins before the search walks up"
    );
  }

  #[test]
  fn directory_ignores_a_declaration_beside_it() {
    let root: PathBuf = install("directory_mode");

    let plan = XrayMountMode::Directory.plan(&root).expect("plan");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get_mounts()[0].origin, "root");
  }

  #[test]
  fn installation_is_an_error_when_none_is_declared() {
    let root: PathBuf = install("installation_missing");

    assert!(XrayMountMode::Installation.plan(root.join("gamedata")).is_err());
  }

  #[test]
  fn parses_and_renders_its_names() {
    for mode in [
      XrayMountMode::Auto,
      XrayMountMode::Directory,
      XrayMountMode::Volumes,
      XrayMountMode::Installation,
      XrayMountMode::ContainingInstallation,
    ] {
      assert_eq!(
        XrayMountMode::try_from(mode.to_string().as_str()).expect("round trip"),
        mode
      );
    }

    assert!(XrayMountMode::try_from("nonsense").is_err());
  }
}
