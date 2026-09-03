//! Holds the format-neutral configuration boundary: two serializations, one payload, one publication rule.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfError;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::archive_pack_config_format::ArchivePackConfigFormat;
use crate::pack::archive_pack_config_json::ArchivePackConfigJson;

/// An empty directory of one case's own, since several assertions here scan one.
fn test_directory(name: &str) -> PathBuf {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("pack/config_codec/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("test directory is created");

  directory
}

fn directory(path: &str, is_recursive: bool) -> ArchivePackDirectory {
  ArchivePackDirectory {
    path: path.into(),
    is_recursive,
  }
}

fn blank() -> ArchivePackConfig {
  ArchivePackConfig::new("gamedata", "db", "configs")
}

fn populated() -> ArchivePackConfig {
  let mut config: ArchivePackConfig = blank();

  config.exclude_extensions = vec![String::from("*.txt"), String::from("*.json")];
  config.include_files = vec![String::from("gamemtl.xr"), String::from("shaders.xr")];
  config.include_directories = vec![directory("configs", true), directory("spawns", false)];
  config.exclude_directories = vec![directory("levels\\build", true)];
  config.header = Some(String::from(
    "[header]\r\nauto_load = true\r\nentry_point = $fs_root$\\gamedata\\\r\n",
  ));

  config
}

/// Export to `name`, then import the result over a blank configuration, as the editor does.
fn round_trip(directory: &Path, config: &ArchivePackConfig, name: &str) -> ArchivePackConfig {
  let path: PathBuf = directory.join(name);

  config.write_config_to_path(&path).expect("configuration is written");

  blank().with_config_file(&path).expect("configuration is read back")
}

#[test]
fn selects_the_codec_from_the_extension_without_case() {
  for name in ["pack.ltx", "pack.LTX", "PACK.Ltx"] {
    assert_eq!(
      ArchivePackConfigFormat::from_path(name).expect("ltx is recognized"),
      ArchivePackConfigFormat::Ltx,
      "{name}"
    );
  }

  for name in ["pack.json", "pack.JSON", "PACK.Json"] {
    assert_eq!(
      ArchivePackConfigFormat::from_path(name).expect("json is recognized"),
      ArchivePackConfigFormat::Json,
      "{name}"
    );
  }
}

#[test]
fn refuses_a_path_whose_extension_names_no_format() {
  // Never guessed from contents: a configuration is a file a person named, so the wrong name is worth reporting.
  let unsupported: XrfError = ArchivePackConfigFormat::from_path("pack.txt").expect_err("txt is not a format");
  let missing: XrfError = ArchivePackConfigFormat::from_path("pack").expect_err("an extension is required");

  for error in [&unsupported, &missing] {
    let message: String = error.to_string();

    assert!(message.contains(".ltx"), "both formats are named: {message}");
    assert!(message.contains(".json"), "both formats are named: {message}");
  }
}

#[test]
fn equivalent_ltx_and_json_produce_the_same_file_owned_fields() {
  let directory: PathBuf = test_directory("equivalent");
  let ltx_path: PathBuf = directory.join("pack.ltx");
  let json_path: PathBuf = directory.join("pack.json");

  fs::write(
    &ltx_path,
    "[options]\nexclude_exts = *.txt, *.json\n\n\
     [include_files]\ngamemtl.xr\nshaders.xr\n\n\
     [include_folders]\nconfigs = true\nspawns = false\n\n\
     [exclude_folders]\nlevels\\build = true\n\n\
     [header]\nauto_load = true\nentry_point = $fs_root$\\gamedata\\\n",
  )
  .expect("ltx is written");

  fs::write(
    &json_path,
    r#"{
      "excludeExtensions": ["*.txt", "*.json"],
      "includeFiles": ["gamemtl.xr", "shaders.xr"],
      "includeDirectories": [
        { "path": "configs", "isRecursive": true },
        { "path": "spawns", "isRecursive": false }
      ],
      "excludeDirectories": [{ "path": "levels\\build", "isRecursive": true }],
      "header": [
        { "key": "auto_load", "value": "true" },
        { "key": "entry_point", "value": "$fs_root$\\gamedata\\" }
      ]
    }"#,
  )
  .expect("json is written");

  let from_ltx: ArchivePackConfig = blank().with_config_file(&ltx_path).expect("ltx applies");
  let from_json: ArchivePackConfig = blank().with_config_file(&json_path).expect("json applies");

  // The header is compared as text because the archive stores exactly these bytes as chunk 666.
  assert_eq!(from_ltx.exclude_extensions, from_json.exclude_extensions);
  assert_eq!(from_ltx.include_files, from_json.include_files);
  assert_eq!(from_ltx.include_directories, from_json.include_directories);
  assert_eq!(from_ltx.exclude_directories, from_json.exclude_directories);
  assert_eq!(from_ltx.header, from_json.header);
}

#[test]
fn each_format_round_trips_on_its_own() {
  let directory: PathBuf = test_directory("round_trip");
  let source: ArchivePackConfig = populated();

  for name in ["pack.ltx", "pack.json"] {
    let restored: ArchivePackConfig = round_trip(&directory, &source, name);

    assert_eq!(restored.exclude_extensions, source.exclude_extensions, "{name}");
    assert_eq!(restored.include_files, source.include_files, "{name}");
    assert_eq!(restored.include_directories, source.include_directories, "{name}");
    assert_eq!(restored.exclude_directories, source.exclude_directories, "{name}");
    assert_eq!(restored.header, source.header, "{name}");
  }
}

#[test]
fn leaves_the_per_run_fields_out_of_the_file() {
  // A configuration travels between machines; a source path and a volume name do not.
  let rendered: String = populated().to_json().render().expect("json renders");

  for absent in [
    "source",
    "destination",
    "\"name\"",
    "maxVolumeSize",
    "mode",
    "volumeExtension",
    "isWithSkipList",
  ] {
    assert!(!rendered.contains(absent), "{absent} is not file-owned:\n{rendered}");
  }
}

#[test]
fn writes_readable_deterministic_json_ending_in_a_newline() {
  let once: String = populated().to_json().render().expect("json renders");
  let twice: String = populated().to_json().render().expect("json renders");

  assert_eq!(once, twice, "equal values render equal bytes");
  assert!(once.ends_with("}\n"), "the json ends with a newline:\n{once}");
  assert!(
    once.contains("\n  \"excludeExtensions\""),
    "indented for reading:\n{once}"
  );

  // Declaration order, so a checked-in configuration only changes when its contents do.
  let order: Vec<usize> = [
    "excludeExtensions",
    "includeFiles",
    "includeDirectories",
    "excludeDirectories",
    "header",
  ]
  .iter()
  .map(|field| {
    once
      .find(field)
      .unwrap_or_else(|| panic!("{field} is written:\n{once}"))
  })
  .collect();

  assert!(
    order.windows(2).all(|pair| pair[0] < pair[1]),
    "fields keep their order:\n{once}"
  );
}

#[test]
fn refuses_malformed_json_rather_than_applying_defaults() {
  let directory: PathBuf = test_directory("malformed");

  // A misspelled key would otherwise pack a different archive than the file describes, and say nothing.
  let cases: [(&str, &str); 3] = [
    ("typo.json", r#"{ "includeFile": ["gamemtl.xr"] }"#),
    ("shape.json", r#"{ "includeFiles": "gamemtl.xr" }"#),
    ("syntax.json", r#"{ "includeFiles": ["gamemtl.xr" }"#),
  ];

  for (name, source) in cases {
    let path: PathBuf = directory.join(name);

    fs::write(&path, source).expect("json is written");

    let error: XrfError = blank().with_config_file(&path).expect_err("malformed json is refused");

    assert!(error.to_string().contains(name), "the refusal names the file: {error}");
  }
}

#[test]
fn an_absent_field_leaves_what_the_caller_holds() {
  let directory: PathBuf = test_directory("partial");
  let path: PathBuf = directory.join("pack.json");

  fs::write(&path, r#"{ "includeFiles": ["gamemtl.xr"] }"#).expect("json is written");

  let applied: ArchivePackConfig = populated().with_config_file(&path).expect("json applies");

  assert_eq!(applied.include_files, vec!["gamemtl.xr"], "what it carries is applied");
  assert_eq!(
    applied.include_directories,
    populated().include_directories,
    "what it omits is left alone, the way an absent LTX section is"
  );
  assert_eq!(applied.header, populated().header);
}

#[test]
fn keeps_the_previous_configuration_when_a_write_is_refused() {
  let directory: PathBuf = test_directory("refused");

  for name in ["pack.ltx", "pack.json"] {
    let path: PathBuf = directory.join(name);

    fs::write(&path, b"previous").expect("previous configuration is written");

    // A destination inside a directory that does not exist cannot be staged, in either format.
    let unreachable: PathBuf = directory.join("missing").join(name);

    assert!(populated().write_config_to_path(&unreachable).is_err(), "{name}");
    assert_eq!(fs::read(&path).expect("previous is readable"), b"previous", "{name}");
  }

  let leftovers: Vec<String> = fs::read_dir(&directory)
    .expect("directory is readable")
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .filter(|name| name.contains("xrf-tmp"))
    .collect();

  assert!(leftovers.is_empty(), "left behind: {leftovers:?}");
}

#[test]
fn json_carries_no_header_when_the_configuration_has_none() {
  let mut config: ArchivePackConfig = populated();

  config.header = None;

  let json: ArchivePackConfigJson = config.to_json();

  assert_eq!(json.header, None);
  assert!(!json.render().expect("json renders").contains("header"));
}
