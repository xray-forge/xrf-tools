use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::pack::config::ArchivePackHeaderEntry;
use crate::patch::config::{ArchivePatchConfig, ArchivePatchConfigJson};

/// A configuration with every file-owned field populated and every per-run field distinctive.
fn authored() -> ArchivePatchConfig {
  let mut config: ArchivePatchConfig = ArchivePatchConfig::new("C:\\Games\\Anomaly", "C:\\out", "mypatch");

  config.include = vec![String::from("configs")];
  config.ignore = vec![String::from("configs\\text")];
  config.exclude_extensions = vec![String::from("*.txt")];

  config
}

/// A scratch path under this test's scope.
fn scratch(scope: &str, name: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("patch_config/{scope}"));

  std::fs::create_dir_all(&root).expect("scratch directory");

  root.join(name)
}

#[test]
fn a_round_trip_returns_the_scope_and_the_header() {
  let path: PathBuf = scratch("round_trip", "patch.json");

  authored().write_config_to_path(&path).expect("the file is written");

  let read: ArchivePatchConfig = ArchivePatchConfig::new("", "", "other")
    .with_config_file(&path)
    .expect("the file is read back");

  assert_eq!(read.include, ["configs"]);
  assert_eq!(read.ignore, ["configs\\text"]);
  assert_eq!(read.exclude_extensions, ["*.txt"]);
  assert_eq!(
    ArchivePackHeaderEntry::split(read.header.as_deref().expect("a header")),
    ArchivePackHeaderEntry::split(&crate::pack::config::default_header())
  );
}

#[test]
fn what_the_run_owns_is_neither_written_nor_read_back() {
  // The whole point of the split: a file shared between machines cannot name someone else's install directory.
  let path: PathBuf = scratch("run_owned", "patch.json");

  authored().write_config_to_path(&path).expect("the file is written");

  let text: String = std::fs::read_to_string(&path).expect("the file reads");

  for absent in ["input", "target", "destination", "name", "mode"] {
    assert!(!text.contains(absent), "'{absent}' is a per-run field and was written");
  }

  let held: ArchivePatchConfig = ArchivePatchConfig::new("C:\\mine", "C:\\mine\\out", "kept")
    .with_config_file(&path)
    .expect("the file is read back");

  assert_eq!(held.input, PathBuf::from("C:\\mine"));
  assert_eq!(held.name, "kept");
}

#[test]
fn an_absent_field_leaves_what_the_caller_holds() {
  // Importing layers rather than replaces, so an explicit option keeps winning over a file.
  let mut held: ArchivePatchConfig = ArchivePatchConfig::new("C:\\Games\\Anomaly", "C:\\out", "patch");

  held.ignore = vec![String::from("kept")];

  let layered: ArchivePatchConfig = held.with_json(&ArchivePatchConfigJson {
    include: Some(vec![String::from("configs")]),
    ..ArchivePatchConfigJson::default()
  });

  assert_eq!(layered.include, ["configs"], "what the file named was applied");
  assert_eq!(layered.ignore, ["kept"], "what it did not name was left alone");
}

#[test]
fn an_empty_collection_is_written_as_absent_rather_than_as_an_empty_list() {
  let empty: ArchivePatchConfigJson = ArchivePatchConfig::new("C:\\in", "C:\\out", "patch").to_json();

  assert_eq!(empty.include, None);
  assert_eq!(empty.ignore, None);
  assert!(empty.header.is_some(), "the default header is a value, not an absence");
}

#[test]
fn an_unknown_key_is_refused_rather_than_ignored() {
  // A file whose `ignore` was typed `ignored` would otherwise compare something other than what it describes.
  let error: String = ArchivePatchConfigJson::parse(br#"{ "ignored": ["configs"] }"#)
    .expect_err("an unknown key is refused")
    .to_string();

  assert!(error.contains("ignored"), "'{error}' names the key it did not know");
}

#[test]
fn a_path_naming_no_supported_format_is_refused_rather_than_sniffed() {
  let path: PathBuf = scratch("bad_extension", "patch.txt");

  std::fs::write(&path, b"{}").expect("the file is written");

  let error: String = ArchivePatchConfig::new("C:\\in", "C:\\out", "patch")
    .with_config_file(&path)
    .expect_err("an unnamed format is refused")
    .to_string();

  assert!(
    error.contains("supported format"),
    "'{error}' says the extension is wrong"
  );
}

#[test]
fn an_ltx_round_trip_returns_the_same_scope_and_header() {
  // The second format exists for consistency with the packer, so the two must agree on what a file carries.
  let path: PathBuf = scratch("ltx_round_trip", "patch.ltx");

  authored().write_config_to_path(&path).expect("the file is written");

  let read: ArchivePatchConfig = ArchivePatchConfig::new("C:\\mine", "C:\\mine\\out", "kept")
    .with_config_file(&path)
    .expect("the file is read back");

  assert_eq!(read.include, ["configs"]);
  assert_eq!(read.ignore, ["configs\\text"]);
  assert_eq!(read.exclude_extensions, ["*.txt"]);
  assert_eq!(
    read.input,
    PathBuf::from("C:\\mine"),
    "the run's own fields are untouched"
  );
  assert_eq!(read.name, "kept");
}

#[test]
fn both_formats_carry_the_same_payload() {
  let json: PathBuf = scratch("both_formats", "patch.json");
  let ltx: PathBuf = scratch("both_formats", "patch.ltx");

  authored().write_config_to_path(&json).expect("json is written");
  authored().write_config_to_path(&ltx).expect("ltx is written");

  let blank = || ArchivePatchConfig::new("C:\\in", "C:\\out", "patch");
  let from_json: ArchivePatchConfig = blank().with_config_file(&json).expect("json reads");
  let from_ltx: ArchivePatchConfig = blank().with_config_file(&ltx).expect("ltx reads");

  assert_eq!(from_json.include, from_ltx.include);
  assert_eq!(from_json.ignore, from_ltx.ignore);
  assert_eq!(from_json.exclude_extensions, from_ltx.exclude_extensions);
  assert_eq!(from_json.header, from_ltx.header);
}
