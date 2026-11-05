use std::fs;
use std::path::PathBuf;

use xrf_test_utils::GeneratedResources;

use crate::FsgameFile;

/// The shape a real `fsgame.ltx` has, trimmed to the aliases these tests reason about.
const CONTENTS: &str = "\
;abbreviation           = recurs| notif | root           | add        | ext         | description
$app_data_root$         = true  | false | $fs_root$      | appdata\\
$arch_dir$              = false | false | $fs_root$      | db\\
$arch_dir_textures$     = false | false | $arch_dir$     | textures\\
$game_data$             = true  | true  | $fs_root$      | gamedata\\
$game_meshes$           = true  | true  | $game_data$    | meshes\\     | *.ogf;*.omf | Game Object files
$downloads$             = false | false | $app_data_root$
";

/// A platform-neutral installation root.
///
/// Alias resolution builds paths with `PathBuf::push`, which inserts the platform separator — so a `C:\install\gamedata`
/// literal would compare unequal to the pushed result on Linux while passing on Windows. These tests are about alias
/// chains, not host path syntax.
fn root() -> PathBuf {
  PathBuf::from("install")
}

fn file() -> FsgameFile {
  FsgameFile::parse(root(), CONTENTS).expect("contents parse")
}

/// An installation holding each of `names` as an fsgame file, scoped by name since tests share the scratch root.
fn installation(name: &str, names: &[&str]) -> PathBuf {
  let root: PathBuf = GeneratedResources::create_dir(&format!("fsgame_file/{name}")).expect("installation root");

  for name in names {
    fs::write(root.join(name), CONTENTS).expect("fsgame file");
  }

  root
}

#[test]
fn finds_the_fsgame_file_each_edition_names() {
  for name in FsgameFile::FILE_NAMES {
    let root: PathBuf = installation(&format!("named/{}", name.trim_end_matches(".ltx")), &[name]);

    assert_eq!(FsgameFile::find(&root), Some(root.join(name)));
    assert_eq!(
      FsgameFile::read(&root).expect("fsgame file reads").get_root(),
      root.as_path()
    );
  }
}

#[test]
fn prefers_fsgame_ltx_over_an_enhanced_edition_name() {
  let root: PathBuf = installation("both", &["fsgame_cop.ltx", "fsgame.ltx"]);

  assert_eq!(FsgameFile::find(&root), Some(root.join("fsgame.ltx")));
}

#[test]
fn finds_nothing_in_a_directory_without_an_fsgame_file() {
  let root: PathBuf = installation("none", &["fsgame_backup.ltx"]);

  assert_eq!(FsgameFile::find(&root), None);
  assert!(FsgameFile::read(&root).is_err());
}

#[test]
fn ignores_a_directory_named_like_an_fsgame_file() {
  let root: PathBuf = installation("directory", &[]);

  fs::create_dir_all(root.join("fsgame.ltx")).expect("directory");

  assert_eq!(FsgameFile::find(&root), None);
}

#[test]
fn keeps_declarations_in_the_order_the_file_lists_them() {
  // Order is the override rule: `$arch_dir$` before `$game_data$` is what makes loose files win.
  let file: FsgameFile = file();
  let aliases: Vec<&str> = file.get_declarations().iter().map(|it| it.alias.as_str()).collect();

  assert_eq!(
    aliases,
    vec![
      "$app_data_root$",
      "$arch_dir$",
      "$arch_dir_textures$",
      "$game_data$",
      "$game_meshes$",
      "$downloads$"
    ]
  );
}

#[test]
fn resolves_an_alias_against_the_installation_root() {
  assert_eq!(file().resolve("$game_data$"), Some(root().join("gamedata")));
}

#[test]
fn resolves_a_chain_of_aliases() {
  assert_eq!(
    file().resolve("$arch_dir_textures$"),
    Some(root().join("db").join("textures"))
  );
  assert_eq!(
    file().resolve("$game_meshes$"),
    Some(root().join("gamedata").join("meshes"))
  );
}

#[test]
fn an_alias_that_only_names_its_root_resolves_to_that_root() {
  assert_eq!(file().resolve("$downloads$"), Some(root().join("appdata")));
}

#[test]
fn answers_none_for_an_undeclared_alias() {
  assert_eq!(file().resolve("$nonexistent$"), None);
}

#[test]
fn resolves_a_root_column_naming_an_absolute_directory() {
  // The engine's locator takes a root no alias names as the directory it spells, so an absolute `gamedata` is the same
  // declaration as one reached through `$fs_root$`, and so is an alias declared below it.
  let gamedata: PathBuf = installation("absolute", &[]).join("gamedata");
  let gamedata_text: &str = gamedata.to_str().expect("a Unicode scratch path");
  let file: FsgameFile = FsgameFile::parse(
    root(),
    &format!(
      "$game_data$ = true | true | {gamedata_text}\n$content_root$ = false | false | {}\n$chained$ = true | true | \
       $content_root$ | gamedata\\\n",
      gamedata
        .parent()
        .and_then(|parent| parent.to_str())
        .expect("a Unicode parent")
    ),
  )
  .expect("contents parse");

  assert_eq!(file.resolve("$game_data$"), Some(gamedata.clone()));
  assert_eq!(file.resolve("$chained$"), Some(gamedata));
}

#[test]
fn resolves_a_relative_root_column_under_the_files_directory() {
  // The locator opens the file from its working directory and takes `$fs_root$` from there, so a relative root names a
  // directory beside the file: the SDK's `fs.ltx` declares `$server_data_root$` as `rawdata\`.
  let file: FsgameFile = FsgameFile::parse(
    root(),
    "$server_data_root$ = false | false | rawdata\\\n$objects$ = true | true | $server_data_root$ | objects\\\n\
     $nested$ = false | false | a/b\\c\\\n$here$ = false | true | .\\\n",
  )
  .expect("contents parse");

  assert_eq!(file.resolve("$server_data_root$"), Some(root().join("rawdata")));
  assert_eq!(file.resolve("$objects$"), Some(root().join("rawdata").join("objects")));
  assert_eq!(
    file.resolve("$nested$"),
    Some(root().join("a").join("b").join("c")),
    "either separator splits a segment, whatever the host"
  );
  assert_eq!(
    file.resolve("$here$"),
    Some(root()),
    "IX-Ray's `fs.ltx` names its own directory `.\\`"
  );
}

#[test]
fn leaves_a_root_column_rooted_without_a_drive_unresolved() {
  // `\` is the root of whichever drive the locator ran on, which a file read elsewhere cannot know.
  let file: FsgameFile = FsgameFile::parse(
    root(),
    "$server_root$ = false | true | \\\n$app_data_root$ = true | false | $server_root$ | _appdata_\\\n",
  )
  .expect("contents parse");

  assert_eq!(file.resolve("$server_root$"), None);
  assert_eq!(file.resolve("$app_data_root$"), None);
}

#[test]
fn a_cycle_terminates_instead_of_recursing() {
  let file: FsgameFile = FsgameFile::parse(
    root(),
    "$first$ = false | false | $second$ | a\\\n$second$ = false | false | $first$ | b\\\n",
  )
  .expect("contents parse");

  assert_eq!(file.resolve("$first$"), None);
}

#[test]
fn resolves_every_declared_alias() {
  let file: FsgameFile = file();

  assert_eq!(file.get_declarations().len(), 6);

  for declaration in file.get_declarations() {
    assert!(
      file.resolve(&declaration.alias).is_some(),
      "every alias of a well-formed file resolves: {}",
      declaration.alias
    );
  }

  assert_eq!(file.resolve("$arch_dir$"), Some(root().join("db")));
}

#[test]
fn rejects_contents_declaring_nothing() {
  assert!(
    FsgameFile::parse(
      root(),
      ";only a comment
"
    )
    .is_err()
  );
}
