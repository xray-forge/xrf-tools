//! Holds the configuration round trip: what the editor writes must read back as what it held.

use xrf_ltx::Ltx;

use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};

fn directory(path: &str, is_recursive: bool) -> ArchivePackDirectory {
  ArchivePackDirectory {
    path: path.into(),
    is_recursive,
  }
}

/// Render a config as the configuration file the editor would export.
fn write_config(config: &ArchivePackConfig) -> String {
  let mut buffer: Vec<u8> = Vec::new();

  config.to_ltx().write_to(&mut buffer).expect("ltx writes");

  String::from_utf8(buffer).expect("ltx is utf8")
}

/// Send a config through LTX and back, the way export followed by import does.
fn round_trip(config: &ArchivePackConfig) -> ArchivePackConfig {
  let written: String = write_config(config);

  ArchivePackConfig::new("gamedata", "db", "configs")
    .with_ltx(&Ltx::read_from_str(&written).unwrap_or_else(|error| panic!("written ltx parses: {error}\n{written}")))
    .expect("written ltx applies")
}

fn populated() -> ArchivePackConfig {
  let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

  config.exclude_extensions = vec![String::from("*.txt"), String::from("*.json")];
  config.include_files = vec![String::from("gamemtl.xr"), String::from("shaders.xr")];
  config.include_directories = vec![directory("configs", true), directory("spawns", false)];
  config.exclude_directories = vec![directory("levels\\build", true)];
  config.header = Some(String::from(
    "[header]\r\nauto_load = true\r\nentry_point = $fs_root$\\gamedata\\\r\n",
  ));

  config
}

#[test]
fn selection_rules_survive_a_round_trip() {
  let restored: ArchivePackConfig = round_trip(&populated());

  assert_eq!(restored.exclude_extensions, vec!["*.txt", "*.json"]);
  assert_eq!(restored.include_files, vec!["gamemtl.xr", "shaders.xr"]);
  assert_eq!(restored.include_directories.len(), 2);
  assert_eq!(restored.include_directories[0].path, "configs");
  assert!(restored.include_directories[0].is_recursive);
  assert_eq!(restored.include_directories[1].path, "spawns");
  assert!(!restored.include_directories[1].is_recursive);
  assert_eq!(restored.exclude_directories[0].path, "levels\\build");
}

#[test]
fn the_header_survives_a_round_trip() {
  let header: String = round_trip(&populated()).header.expect("header is carried");

  // The engine reads the mount point out of this, so losing it in an export would break the archive.
  assert!(header.contains("entry_point = $fs_root$\\gamedata\\"));
  assert!(header.contains("auto_load = true"));
}

#[test]
fn the_packed_root_survives_as_itself() {
  let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

  config.include_directories = vec![directory("", false)];

  // An empty path means the packed root, which the dialect spells `.\` and must not become a literal.
  assert!(write_config(&config).contains(".\\"));
  assert_eq!(round_trip(&config).include_directories[0].path, "");
}

#[test]
fn an_empty_config_writes_nothing_to_mislead_a_reader() {
  let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");
  let written: String = write_config(&config);

  assert!(
    !written.contains("include_folders"),
    "no sections are invented: {written}"
  );
  assert!(round_trip(&config).include_directories.is_empty());
}

#[test]
fn a_fresh_config_mounts_where_gamedata_belongs() {
  let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");
  let header: &str = config.header.as_deref().expect("a header by default");

  // Defaulting to none would hand the engine an archive it reads as an encrypted ShoC one.
  assert!(header.contains("entry_point = $fs_root$\\gamedata\\"));
  assert!(header.contains("auto_load = true"));
  assert!(write_config(&config).contains("entry_point"));
}

#[test]
fn a_configured_header_replaces_the_default() {
  let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs")
    .with_ltx(&Ltx::read_from_str("[header]\nentry_point = $fs_root$\\levels\\\n").expect("ltx parses"))
    .expect("ltx applies");
  let header: &str = config.header.as_deref().expect("header is carried");

  assert!(header.contains("entry_point = $fs_root$\\levels\\"));
  assert!(!header.contains("gamedata"), "the default is replaced, not merged");
}

#[test]
fn a_second_round_trip_changes_nothing() {
  let once: ArchivePackConfig = round_trip(&populated());
  let twice: ArchivePackConfig = round_trip(&once);

  assert_eq!(write_config(&once), write_config(&twice));
}
