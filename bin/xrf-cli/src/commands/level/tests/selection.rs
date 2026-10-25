//! How a command of the domain is told which level to read.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayMountMode, XrayRoots, XraySearch};

use crate::commands::level::level_assets::LevelAssets;

/// A generated root declares no installation, so name the mode rather than letting `Auto` search upward.
fn search(root: &Path) -> XrfResult<XraySearch> {
  XrayRoots::one(root.to_path_buf(), XrayMountMode::Directory)
    .to_search_plan()?
    .open()
}

/// A root holding the named levels, each with a `level` file and a `level.geom` beside it.
fn create_root(name: &str, levels: &[&str]) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("level/selection/{name}"))?;

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
fn reads_a_level_of_mounted_roots_by_name() -> XrfResult {
  let root: PathBuf = create_root("by_name", &["jupiter", "zaton"])?;
  let assets: LevelAssets = LevelAssets::open_asset(search(&root)?, "zaton")?;

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
fn a_name_no_level_answers_to_is_refused() -> XrfResult {
  let root: PathBuf = create_root("unknown_name", &["jupiter"])?;
  let search: XraySearch = search(&root)?;

  assert!(
    LevelAssets::open_asset(search.clone(), "pripyat").is_err(),
    "a level the roots do not hold is refused rather than read as empty"
  );
  assert!(
    LevelAssets::open_asset(search, "not_a_level").is_err(),
    "a directory without a bundle is not a level"
  );

  Ok(())
}

#[test]
fn lists_only_the_names_that_are_levels() -> XrfResult {
  let root: PathBuf = create_root("listing", &["jupiter", "zaton", "l01_escape"])?;
  assert_eq!(
    LevelAssets::list_names(&search(&root)?)?,
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
fn reads_a_level_directory_as_it_sits_on_disk() -> XrfResult {
  let root: PathBuf = create_root("directory", &["jupiter"])?;
  let assets: LevelAssets = LevelAssets::open_directory(&root.join("levels").join("jupiter"));

  assert_eq!(assets.read("level")?.as_deref(), Some(b"bundle".as_slice()));
  assert_eq!(assets.read("level.geomx")?, None);

  Ok(())
}
