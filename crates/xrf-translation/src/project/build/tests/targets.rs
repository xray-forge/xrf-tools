use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};

use crate::language::TranslationLanguage;
use crate::project::build::options::ProjectBuildOptions;
use crate::project::build::run::build_dir;
use crate::project::build::targets::{target_path, validate_targets};

/// The one source every target test builds from; what it contains is never the point.
const SOURCE_JSON: &str = r#"{"st_test":{"eng":"text"}}"#;

fn options(path: PathBuf, output_dir: PathBuf) -> ProjectBuildOptions {
  ProjectBuildOptions {
    is_sorted: false,
    output: xrf_output::OutputOptions::default(),
    language: TranslationLanguage::English,
    path,
    output_dir,
  }
}

#[test]
fn a_json_stem_becomes_the_gamedata_file_name() -> XrfResult {
  let source_root = PathBuf::from("translations");
  let output_root = PathBuf::from("output");
  let options = options(source_root.clone(), output_root.clone());

  let target = target_path(
    &source_root.join("ui").join("st_ui.json"),
    &output_root,
    &TranslationLanguage::English,
    &options,
  )?;

  // The destination directory carries the language, so the stem is all the name that is needed.
  assert_eq!(target, output_root.join("eng").join("ui").join("st_ui.xml"));

  // A dot in the stem is part of the name, not a language suffix: nothing strips it any more.
  let dotted = target_path(
    &source_root.join("ui").join("st_items.eng.json"),
    &output_root,
    &TranslationLanguage::English,
    &options,
  )?;

  assert_eq!(dotted, output_root.join("eng").join("ui").join("st_items.eng.xml"));

  Ok(())
}

#[test]
fn directory_builds_preserve_relative_source_paths() -> XrfResult {
  let test_root: &str = "build_targets/relative_paths";
  let source_root = build_absolute_generated_test_resource_path(&format!("{test_root}/source"));
  let output_root = build_absolute_generated_test_resource_path(&format!("{test_root}/output"));

  if output_root.exists() {
    fs::remove_dir_all(&output_root)?;
  }

  for relative_path in ["one/common.json", "two/common.json"] {
    write_generated_test_resource(&format!("{test_root}/source/{relative_path}"), SOURCE_JSON)?;
  }

  build_dir(&source_root, &options(source_root.clone(), output_root.clone()))?;

  // Two files of the same name in different directories must not land on top of each other.
  assert!(output_root.join("eng/one/common.xml").is_file());
  assert!(output_root.join("eng/two/common.xml").is_file());

  Ok(())
}

#[test]
fn builds_reject_two_sources_that_would_write_one_target() -> XrfResult {
  // Checked against the target rule directly rather than through a directory of files: now that JSON
  // is the only source format, the sole way two of them collide is by differing in case alone, and a
  // case-insensitive host cannot hold both to be walked in the first place.
  let source_root = PathBuf::from("translations");
  let output_root = PathBuf::from("output");
  let options = options(source_root.clone(), output_root.clone());

  let error = validate_targets(
    &[source_root.join("Common.json"), source_root.join("common.json")],
    &options,
  )
  .unwrap_err();

  assert!(error.to_string().contains("both build to"));

  Ok(())
}

#[test]
fn directory_builds_reject_output_inside_the_source_tree() -> XrfResult {
  let test_root: &str = "build_targets/output_inside_source";
  let source_root = build_absolute_generated_test_resource_path(&format!("{test_root}/source"));

  write_generated_test_resource(&format!("{test_root}/source/common.json"), SOURCE_JSON)?;

  let output_dir = source_root.join("built");
  let error = build_dir(&source_root, &options(source_root.clone(), output_dir)).unwrap_err();

  assert!(error.to_string().contains("must be outside source directory"));

  Ok(())
}
