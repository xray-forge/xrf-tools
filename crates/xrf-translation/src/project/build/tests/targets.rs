use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};

use crate::language::TranslationLanguage;
use crate::project::build::options::ProjectBuildOptions;
use crate::project::build::run::build_dir;
use crate::project::build::targets::target_path;

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
fn language_suffixed_xml_uses_the_gamedata_file_name() -> XrfResult {
  let source_root = PathBuf::from("translations");
  let output_root = PathBuf::from("output");
  let options = options(source_root.clone(), output_root.clone());

  let target = target_path(
    &source_root.join("ui").join("st_ui.eng.xml"),
    &output_root,
    &TranslationLanguage::English,
    &options,
  )?;

  assert_eq!(target, output_root.join("eng").join("ui").join("st_ui.xml"));

  let json_target = target_path(
    &source_root.join("ui").join("st_items.eng.json"),
    &output_root,
    &TranslationLanguage::English,
    &options,
  )?;

  assert_eq!(json_target, output_root.join("eng").join("ui").join("st_items.eng.xml"));

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
fn directory_builds_reject_colliding_json_and_xml_targets() -> XrfResult {
  let test_root: &str = "build_targets/colliding";
  let source_root = build_absolute_generated_test_resource_path(&format!("{test_root}/source"));
  let output_root = build_absolute_generated_test_resource_path(&format!("{test_root}/output"));

  write_generated_test_resource(&format!("{test_root}/source/common.json"), SOURCE_JSON)?;
  write_generated_test_resource(
    &format!("{test_root}/source/common.xml"),
    "<string_table></string_table>",
  )?;

  let error = build_dir(&source_root, &options(source_root.clone(), output_root)).unwrap_err();

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
