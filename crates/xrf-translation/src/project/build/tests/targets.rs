use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::language::TranslationLanguage;
use crate::project::build::options::ProjectBuildOptions;
use crate::project::build::run::build_roots;
use crate::project::build::targets::{ensure_output_outside_roots, target_path, validate_targets};

/// The one source every target test builds from; what it contains is never the point.
const SOURCE_JSON: &str = r#"{"st_test":{"eng":"text"}}"#;

fn options(output_dir: PathBuf) -> ProjectBuildOptions {
  ProjectBuildOptions {
    job: Default::default(),
    is_sorted: false,
    output: xrf_output::OutputOptions::default(),
    language: TranslationLanguage::English,
    output_dir,
  }
}

fn roots(root: &str) -> XrayRoots {
  XrayRoots::one(
    build_absolute_generated_test_resource_path(root).display().to_string(),
    XrayMountMode::Directory,
  )
}

#[test]
fn a_json_stem_becomes_the_gamedata_file_name() -> XrfResult {
  let output_root = PathBuf::from("output");

  let target = target_path(r"ui\st_ui.json", &output_root, &TranslationLanguage::English)?;

  // The destination directory carries the language, so the stem is all the name that is needed, and
  // the logical path below the root is mirrored into it.
  assert_eq!(target, output_root.join("eng").join("ui").join("st_ui.xml"));

  // A dot in the stem is part of the name, not a language suffix: nothing strips it any more.
  let dotted = target_path(r"ui\st_items.eng.json", &output_root, &TranslationLanguage::English)?;

  assert_eq!(dotted, output_root.join("eng").join("ui").join("st_items.eng.xml"));

  Ok(())
}

#[test]
fn directory_builds_preserve_relative_source_paths() -> XrfResult {
  let test_root: &str = "build_targets/relative_paths";
  let output_root = build_absolute_generated_test_resource_path(&format!("{test_root}/output"));

  if output_root.exists() {
    fs::remove_dir_all(&output_root)?;
  }

  for relative_path in ["one/common.json", "two/common.json"] {
    write_generated_test_resource(&format!("{test_root}/source/{relative_path}"), SOURCE_JSON)?;
  }

  let result = build_roots(
    &roots(&format!("{test_root}/source")),
    None,
    &options(output_root.clone()),
  )?;

  // Two files of the same name in different directories must not land on top of each other.
  assert!(output_root.join("eng/one/common.xml").is_file());
  assert!(output_root.join("eng/two/common.xml").is_file());

  assert_eq!(result.sources, 2);
  assert_eq!(result.files, 2);
  assert_eq!(result.languages.len(), 1);
  assert_eq!(result.languages[0].language, "eng");
  assert_eq!(result.languages[0].files, 2);

  Ok(())
}

#[test]
fn builds_report_one_row_per_language() -> XrfResult {
  let test_root: &str = "build_targets/languages";
  let output_root = build_absolute_generated_test_resource_path(&format!("{test_root}/output"));

  if output_root.exists() {
    fs::remove_dir_all(&output_root)?;
  }

  write_generated_test_resource(&format!("{test_root}/source/st_a.json"), SOURCE_JSON)?;

  let mut all = options(output_root.clone());

  all.language = TranslationLanguage::All;

  let result = build_roots(&roots(&format!("{test_root}/source")), None, &all)?;

  // One string table per language, whatever the source carries: a missing translation compiles to the
  // id itself, which is the engine's own fallback.
  assert_eq!(result.sources, 1);
  assert_eq!(result.files, 8);
  assert_eq!(result.languages.len(), 8);
  assert!(result.languages.iter().all(|summary| summary.files == 1));
  assert!(result.languages.iter().all(|summary| summary.entries == 1));

  Ok(())
}

#[test]
fn builds_reject_two_sources_that_would_write_one_target() -> XrfResult {
  // Checked against the target rule directly rather than through a tree of files: now that JSON is the
  // only source format, the sole way two of them collide is by differing in case alone, and a
  // case-insensitive host cannot hold both to be walked in the first place.
  let error = validate_targets(
    &[String::from("Common.json"), String::from("common.json")],
    &options(PathBuf::from("output")),
  )
  .unwrap_err();

  assert!(error.to_string().contains("both build to"));

  Ok(())
}

#[test]
fn builds_reject_output_inside_any_source_root() -> XrfResult {
  // Every loose root is checked, not just the first: a layered build reads from all of them, and
  // filling an authored tree with generated tables is nobody's intention whichever one it is.
  let test_root: &str = "build_targets/output_inside_source";
  let source_root = build_absolute_generated_test_resource_path(&format!("{test_root}/source"));
  let other_root = build_absolute_generated_test_resource_path(&format!("{test_root}/other"));

  write_generated_test_resource(&format!("{test_root}/source/common.json"), SOURCE_JSON)?;
  write_generated_test_resource(&format!("{test_root}/other/common.json"), SOURCE_JSON)?;

  let layered: XrayRoots = XrayRoots::new([
    xrf_vfs::XrayRoot::new(other_root.display().to_string(), XrayMountMode::Directory),
    xrf_vfs::XrayRoot::new(source_root.display().to_string(), XrayMountMode::Directory),
  ]);

  let error = ensure_output_outside_roots(&layered, &source_root.join("built")).unwrap_err();

  assert!(error.to_string().contains("must be outside source directory"));

  Ok(())
}
