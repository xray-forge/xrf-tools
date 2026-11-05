use std::fs;
use std::path::PathBuf;

use xrf_test_utils::GeneratedResources;

use crate::{FsgameFile, XraySdkInstallation, XraySearch};

/// The shape the CoP SDK 0.7's `editors\fs.ltx` has, trimmed to the aliases these tests reason about: rawdata and
/// gamedata named by relative roots, and rawdata's own textures beside gamedata's.
const CONTENTS: &str = "\
;abbreviation          = recurs| notif| root|                add|        ext|         description
$sdk_root$             = false| false| $fs_root$
$server_data_root$     = false| false| rawdata\\
$server_root$          = false| true|  \\
$game_data$            = false| true|  gamedata\\
$game_textures$        = true|  true|  $game_data$|         textures\\
$textures$             = true|  true|  $server_data_root$|  textures\\
$objects$              = true|  true|  $server_data_root$|  objects\\|   *.object;*.lwo| XRay objects
$maps$                 = true|  true|  $server_data_root$|  levels\\|    *.level|        XRay levels
$import$               = true|  true|  import\\|            |            *.object;*.lwo| Files
";

/// An SDK editors' directory holding `fs.ltx`, a rawdata object and texture, and a gamedata texture of the same name.
fn editors(name: &str) -> PathBuf {
  let root: PathBuf = GeneratedResources::create_dir(&format!("xray_sdk_installation/{name}")).expect("editors root");

  for file in [
    "rawdata/objects/editor/axis.object",
    "rawdata/textures/ed/ed_axis.tga",
    "gamedata/textures/ed/ed_axis.dds",
  ] {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("a parent")).expect("tree");
    fs::write(path, b"x").expect("file");
  }

  fs::write(root.join(XraySdkInstallation::FILE_NAME), CONTENTS).expect("fs.ltx");

  root
}

#[test]
fn reads_the_layout_an_editors_directory_declares() {
  let root: PathBuf = editors("read");
  let sdk: XraySdkInstallation = XraySdkInstallation::read(&root).expect("fs.ltx reads");

  assert_eq!(XraySdkInstallation::find(&root), Some(root.join("fs.ltx")));
  assert_eq!(sdk.get_rawdata_root(), Some(root.join("rawdata")));
  assert_eq!(sdk.get_file().resolve("$game_data$"), Some(root.join("gamedata")));
  assert_eq!(sdk.get_file().resolve("$import$"), Some(root.join("import")));
}

#[test]
fn names_each_editor_alias_by_its_path_inside_rawdata() {
  let sdk: XraySdkInstallation = XraySdkInstallation::from(FsgameFile::parse("editors", CONTENTS).expect("parses"));
  let prefix = |alias: &str| sdk.get_rawdata_prefix(alias).expect("a valid logical path");

  assert_eq!(prefix("$objects$").as_deref(), Some("objects"));
  assert_eq!(prefix("$maps$").as_deref(), Some("levels"));
  assert_eq!(prefix("$textures$").as_deref(), Some("textures"));
  assert_eq!(prefix("$server_data_root$").as_deref(), Some(""));
  assert_eq!(
    prefix("$game_textures$"),
    None,
    "gamedata lies beside rawdata, not inside it"
  );
  assert_eq!(prefix("$import$"), None);
  assert_eq!(prefix("$undeclared$"), None);
}

#[test]
fn searches_rawdata_apart_from_gamedata() {
  // Both trees hold `textures\ed\`; a rawdata search answers with the TGA and never with the game's DDS.
  let root: PathBuf = editors("search");
  let search: XraySearch = XraySdkInstallation::read(&root)
    .and_then(|sdk| sdk.to_rawdata_search_plan())
    .and_then(|plan| plan.open())
    .expect("rawdata opens");

  assert!(search.find("objects\\editor\\axis.object").expect("lookup").is_some());
  assert!(search.find("textures\\ed\\ed_axis.tga").expect("lookup").is_some());
  assert!(search.find("textures\\ed\\ed_axis.dds").expect("lookup").is_none());
}

#[test]
fn refuses_to_plan_rawdata_a_layout_does_not_declare() {
  let sdk: XraySdkInstallation = XraySdkInstallation::from(
    FsgameFile::parse("editors", "$game_data$ = false | true | gamedata\\\n").expect("parses"),
  );

  assert_eq!(sdk.get_rawdata_root(), None);
  assert!(sdk.to_rawdata_search_plan().is_err());
}

#[test]
fn finds_no_layout_in_a_directory_without_fs_ltx() {
  let root: PathBuf = GeneratedResources::create_dir("xray_sdk_installation/none").expect("root");

  assert_eq!(XraySdkInstallation::find(&root), None);
  assert!(XraySdkInstallation::read(&root).is_err());
}
