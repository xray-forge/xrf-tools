//! Pins what the packing-configuration commands read and write, and that neither depends on which format a path names.

use std::fs;
use std::future::Future;
use std::path::PathBuf;

use xrf_pack::ArchivePackConfig;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::types::TauriResult;
use crate::plugins::archives::commands::export_pack_config::archives_export_pack_config;
use crate::plugins::archives::commands::import_pack_config::archives_import_pack_config;

/// Drive one command to its answer.
///
/// The commands are declared `async` because that is how they are registered, not because either awaits anything;
/// this runs them the way the IPC layer would rather than adding a test runtime for two synchronous bodies.
fn run<T>(command: impl Future<Output = TauriResult<T>>) -> TauriResult<T> {
  tauri::async_runtime::block_on(command)
}

/// An empty directory of one case's own, since these commands write files.
fn test_directory(name: &str) -> PathBuf {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("archives/pack_config/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("test directory is created");

  directory
}

/// The configuration the editor would hold: per-run fields filled in, so a round trip can prove they stay the form's.
fn open_form() -> ArchivePackConfig {
  let mut config: ArchivePackConfig = ArchivePackConfig::new("C:\\work\\gamedata", "C:\\work\\db", "my_mod");

  config.include_files = vec![String::from("gamemtl.xr")];
  config.exclude_extensions = vec![String::from("*.thm")];

  config
}

fn blank_form() -> ArchivePackConfig {
  ArchivePackConfig::new("", "", "gamedata")
}

#[test]
fn writes_and_reads_back_either_format() {
  let directory: PathBuf = test_directory("round_trip");

  for name in ["pack.ltx", "pack.json"] {
    let path: String = directory.join(name).display().to_string();

    run(archives_export_pack_config(&path, open_form())).unwrap_or_else(|error| panic!("{name} is exported: {error}"));

    // Imported over a blank form, the way the editor layers a file onto whatever it already holds.
    let imported: ArchivePackConfig = run(archives_import_pack_config(&path, blank_form()))
      .unwrap_or_else(|error| panic!("{name} is imported: {error}"));

    assert_eq!(imported.include_files, open_form().include_files, "{name}");
    assert_eq!(imported.exclude_extensions, open_form().exclude_extensions, "{name}");
    assert_eq!(imported.header, open_form().header, "{name}");

    // The file carries the rules; the form keeps the paths and the volume name it was opened with.
    assert_eq!(imported.source, PathBuf::from(""), "{name}");
    assert_eq!(imported.name, "gamedata", "{name}");
  }
}

#[test]
fn both_formats_carry_the_same_rules() {
  let directory: PathBuf = test_directory("equivalent");
  let ltx: String = directory.join("pack.ltx").display().to_string();
  let json: String = directory.join("pack.json").display().to_string();

  run(archives_export_pack_config(&ltx, open_form())).expect("ltx exports");
  run(archives_export_pack_config(&json, open_form())).expect("json exports");

  let from_ltx: ArchivePackConfig = run(archives_import_pack_config(&ltx, blank_form())).expect("ltx imports");
  let from_json: ArchivePackConfig = run(archives_import_pack_config(&json, blank_form())).expect("json imports");

  assert_eq!(from_ltx.include_files, from_json.include_files);
  assert_eq!(from_ltx.exclude_extensions, from_json.exclude_extensions);
  assert_eq!(from_ltx.include_directories, from_json.include_directories);
  assert_eq!(from_ltx.exclude_directories, from_json.exclude_directories);
  assert_eq!(from_ltx.header, from_json.header);
}

#[test]
fn refuses_a_path_whose_extension_names_no_format() {
  let directory: PathBuf = test_directory("unsupported");
  let path: String = directory.join("pack.txt").display().to_string();

  let exported: String =
    run(archives_export_pack_config(&path, open_form())).expect_err("an unsupported destination is refused");

  assert!(exported.contains(".ltx") && exported.contains(".json"), "{exported}");
  assert!(!PathBuf::from(&path).exists(), "a refused export writes nothing");

  // The contents would parse as LTX; the name is what decides, so the reader is never guessed from bytes.
  fs::write(&path, "[include_files]\ngamemtl.xr\n").expect("a configuration is written under the wrong name");

  let imported: String =
    run(archives_import_pack_config(&path, open_form())).expect_err("an unsupported source is refused");

  assert!(imported.contains(".ltx") && imported.contains(".json"), "{imported}");
}

#[test]
fn keeps_the_previous_configuration_when_an_export_is_refused() {
  let directory: PathBuf = test_directory("refused");

  for name in ["pack.ltx", "pack.json"] {
    let path: PathBuf = directory.join(name);

    fs::write(&path, b"previous").expect("previous configuration is written");

    // A destination inside a directory that does not exist cannot be staged, in either format.
    let unreachable: String = directory.join("missing").join(name).display().to_string();

    assert!(
      run(archives_export_pack_config(&unreachable, open_form())).is_err(),
      "{name}"
    );
    assert_eq!(fs::read(&path).expect("previous is readable"), b"previous", "{name}");
  }
}
