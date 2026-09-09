//! What a written `[header]` has to carry, and what a named key does to the default.

use crate::pack::config::{ArchivePackConfig, ArchivePackHeaderEntry, default_header};

/// One `key=value` pair, as a command line supplies it.
fn entry(key: &str, value: &str) -> ArchivePackHeaderEntry {
  ArchivePackHeaderEntry {
    key: key.to_owned(),
    value: value.to_owned(),
  }
}

/// The refusal a configuration produced, or a panic when it was accepted.
fn refusal(header: Option<&str>) -> String {
  ArchivePackConfig {
    header: header.map(str::to_owned),
    ..ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
  }
  .validate_for_packing()
  .expect_err("the header is refused")
  .to_string()
}

#[test]
fn naming_one_key_adds_it_to_the_default_rather_than_replacing_it() {
  // The defect this exists for: `--header creator=me` used to write a header holding only `creator`, and a volume
  // without `auto_load` stops the engine on load rather than mounting oddly.
  let merged: Vec<ArchivePackHeaderEntry> = ArchivePackHeaderEntry::over_default(&[entry("creator", "\"Modder\"")]);
  let keys: Vec<&str> = merged.iter().map(|entry| entry.key.as_str()).collect();

  assert_eq!(keys, ["auto_load", "entry_point", "creator"]);
  assert_eq!(
    merged[1].value, "$fs_root$\\gamedata\\",
    "the entry point Anomaly's own template marks 'do not change !' survives"
  );
}

#[test]
fn naming_a_default_key_overrides_that_one_and_leaves_the_rest() {
  // GSC's own `build_map.ltx` ships `auto_load = false` for multiplayer maps, so the value is the author's to set.
  let merged: Vec<ArchivePackHeaderEntry> = ArchivePackHeaderEntry::over_default(&[entry("auto_load", "false")]);

  assert_eq!(merged.len(), 2, "overriding a key does not add one");
  assert_eq!(merged[0].value, "false");
  assert_eq!(merged[1].key, "entry_point");
}

#[test]
fn the_default_header_is_loadable_as_written() {
  ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
    .validate_for_packing()
    .expect("the default a run writes without being asked must itself be loadable");
}

#[test]
fn a_header_missing_a_key_the_engine_reads_unconditionally_is_refused() {
  // `ProcessArchive` asks for `auto_load` and `LoadArchive` for `entry_point`, both through `r_string`, which ends in
  // `xrDebug::Fatal` when the key is absent.
  let without_auto_load: String = refusal(Some("[header]\r\nentry_point = $fs_root$\\gamedata\\\r\n"));
  let without_entry_point: String = refusal(Some("[header]\r\nauto_load = true\r\n"));

  assert!(
    without_auto_load.contains("names no 'auto_load'"),
    "'{without_auto_load}' names the missing key"
  );
  assert!(
    without_entry_point.contains("names no 'entry_point'"),
    "'{without_entry_point}' names the missing key"
  );
}

#[test]
fn a_header_with_no_section_is_refused() {
  let message: String = refusal(Some("auto_load = true\r\nentry_point = $fs_root$\\gamedata\\\r\n"));

  assert!(
    message.contains("names no '[header]' section"),
    "'{message}' says the section is missing"
  );
}

#[test]
fn an_entry_point_the_loader_asserts_on_is_refused() {
  // `LoadArchive` special-cases the bare `gamedata` literal and otherwise asserts the value opens with an alias.
  let message: String = refusal(Some("[header]\r\nauto_load = true\r\nentry_point = mydata\\\r\n"));

  assert!(message.contains("is neither 'gamedata' nor an alias"), "'{message}'");
}

#[test]
fn the_bare_gamedata_literal_and_an_alias_are_both_accepted() {
  for value in ["gamedata", "$fs_root$\\gamedata\\", "$fs_root$\\levels\\"] {
    ArchivePackConfig {
      header: Some(format!("[header]\r\nauto_load = true\r\nentry_point = {value}\r\n")),
      ..ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
    }
    .validate_for_packing()
    .unwrap_or_else(|error| panic!("'{value}' is a value the engine can read, but was refused: {error}"));
  }
}

#[test]
fn a_run_asking_for_no_header_at_all_is_still_allowed() {
  // The documented escape hatch, and the one case where nothing is written for the engine to fail to read.
  ArchivePackConfig {
    header: None,
    ..ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
  }
  .validate_for_packing()
  .expect("a headerless volume is a choice, not a malformed header");

  assert!(default_header().contains("entry_point"));
}
