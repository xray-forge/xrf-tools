//! How a command of the domain is told which level to read.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayRoots, XrayVfs};

use crate::commands::level::level_assets::LevelAssets;

/// A generated root declares no installation, so name the mode rather than letting `Auto` search upward.
fn roots(root: &Path) -> XrayRoots {
  XrayRoots::one(root.to_path_buf(), XrayMountMode::Directory)
}

/// A root holding the named levels, each with a `level` file and a `level.geom` beside it.
fn create_root(name: &str, levels: &[&str]) -> XrfResult<PathBuf> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("level/selection/{name}"));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  for level in levels {
    fs::create_dir_all(root.join("levels").join(level))?;
    fs::write(root.join("levels").join(level).join("level"), b"bundle")?;
    fs::write(root.join("levels").join(level).join("level.geom"), b"geometry")?;
  }

  // A directory of lightmaps alone is not a level, and is here to prove the listing says so.
  fs::create_dir_all(root.join("levels").join("not_a_level"))?;
  fs::write(root.join("levels").join("not_a_level").join("lmap#1_1.dds"), b"picture")?;

  Ok(root)
}

#[test]
fn test_reads_a_level_of_mounted_roots_by_name() -> XrfResult {
  let root: PathBuf = create_root("by_name", &["jupiter", "zaton"])?;
  let vfs: XrayVfs = roots(&root).open()?;
  let scope: XrayLookupScope = XrayLookupScope::all();
  let assets: LevelAssets = LevelAssets::open_asset(&vfs, &scope, "zaton")?;

  assert_eq!(assets.read("level")?.as_deref(), Some(b"bundle".as_slice()));
  assert_eq!(assets.read("level.geom")?.as_deref(), Some(b"geometry".as_slice()));
  assert_eq!(
    assets.read("level.geomx")?,
    None,
    "a file the level ships without is absent rather than an error"
  );
  assert_eq!(assets.describe(), "levels\\zaton");
  assert_eq!(assets.describe_file("level.geom"), "levels\\zaton\\level.geom");

  Ok(())
}

#[test]
fn test_a_name_no_level_answers_to_is_refused() -> XrfResult {
  let root: PathBuf = create_root("unknown_name", &["jupiter"])?;
  let vfs: XrayVfs = roots(&root).open()?;
  let scope: XrayLookupScope = XrayLookupScope::all();

  assert!(
    LevelAssets::open_asset(&vfs, &scope, "pripyat").is_err(),
    "a level the roots do not hold is refused rather than read as empty"
  );
  assert!(
    LevelAssets::open_asset(&vfs, &scope, "not_a_level").is_err(),
    "a directory without a bundle is not a level"
  );

  Ok(())
}

#[test]
fn test_lists_only_the_names_that_are_levels() -> XrfResult {
  let root: PathBuf = create_root("listing", &["jupiter", "zaton", "l01_escape"])?;
  let vfs: XrayVfs = roots(&root).open()?;

  assert_eq!(
    LevelAssets::list_names(&vfs, &XrayLookupScope::all())?,
    vec![
      String::from("jupiter"),
      String::from("l01_escape"),
      String::from("zaton")
    ],
    "sorted, and the lightmap directory is not among them"
  );

  Ok(())
}

#[test]
fn test_reads_a_level_directory_as_it_sits_on_disk() -> XrfResult {
  let root: PathBuf = create_root("directory", &["jupiter"])?;
  let assets: LevelAssets = LevelAssets::open_directory(&root.join("levels").join("jupiter"));

  assert_eq!(assets.read("level")?.as_deref(), Some(b"bundle".as_slice()));
  assert_eq!(assets.read("level.geomx")?, None);

  Ok(())
}
