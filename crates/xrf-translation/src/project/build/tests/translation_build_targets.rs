use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::language::translation_language::TranslationLanguage;
use crate::project::build::translation_build_options::TranslationBuildOptions;
use crate::project::build::translation_build_targets::TranslationBuildTargets;
use crate::project::build::translation_builder::TranslationBuilder;

/// The one source every target test builds from; what it contains is never the point.
const SOURCE_JSON: &str = r#"{"st_test":{"eng":"text"}}"#;

fn options(output_dir: PathBuf) -> TranslationBuildOptions {
  TranslationBuildOptions {
    job: Default::default(),
    is_sorted: false,
    output: xrf_output::OutputOptions::default(),
    language: TranslationLanguage::English,
    output_dir,
    link_policy: Default::default(),
  }
}

fn roots(root: &str) -> XrayRoots {
  XrayRoots::one(GeneratedResources::path(root), XrayMountMode::Directory)
}

#[test]
fn a_json_stem_becomes_the_gamedata_file_name_in_the_language_directory() -> XrfResult {
  // The destination directory carries the language, so the stem is all the name that is needed, and the directory the
  // source sits in is dropped: the engine opens a table by its bare name in the language directory.
  let target = TranslationBuildTargets::target_path(r"ui\st_ui.json", TranslationLanguage::English)?;

  assert_eq!(target.as_path(), Path::new("eng").join("st_ui.xml"));

  // A dot in the stem is part of the name, not a language suffix: nothing strips it any more.
  let dotted = TranslationBuildTargets::target_path(r"ui\st_items.eng.json", TranslationLanguage::English)?;

  assert_eq!(dotted.as_path(), Path::new("eng").join("st_items.eng.xml"));

  Ok(())
}

#[test]
fn a_target_never_leaves_the_output_directory() {
  for source in [
    r"..\..\input\st_test.json",
    "../st_test.json",
    r"c:\input\st_test.json",
    "c:st_test.json",
  ] {
    assert!(
      TranslationBuildTargets::target_path(source, TranslationLanguage::English).is_err(),
      "'{source}' must not name a target"
    );
  }
}

/// A sentinel `input/st_test.xml` beside the source, and the output directory a single-file build is pointed at.
fn single_file_layout(test_root: &str) -> XrfResult<(PathBuf, PathBuf)> {
  let root: PathBuf = GeneratedResources::create_dir(test_root)?;

  let source: PathBuf = GeneratedResources::write(&format!("{test_root}/input/st_test.json"), SOURCE_JSON)?;

  GeneratedResources::write(&format!("{test_root}/input/st_test.xml"), "keep me")?;

  Ok((source, root.join("out")))
}

fn assert_built_inside_output(source: &Path, output: &Path) -> XrfResult {
  let result = TranslationBuilder::build_file(&source, &options(output.to_path_buf()))?;

  assert_eq!(result.files, 1);
  assert!(
    output.join("eng").join("st_test.xml").is_file(),
    "built from '{}'",
    source.display()
  );
  assert_eq!(
    fs::read_to_string(source.with_extension("xml"))?,
    "keep me",
    "building '{}' overwrote the file beside its source",
    source.display()
  );

  Ok(())
}

/// The source path says where the file is read from, never where its string table goes: a single-file build writes
/// `<output>/<language>/<file name>.xml` whatever the path held.
#[test]
fn a_single_file_build_writes_inside_the_output_whatever_its_path_holds() -> XrfResult {
  let (source, output) = single_file_layout("build_targets/single_file_paths")?;
  let current: PathBuf = std::env::current_dir()?;

  // Parent-relative: climb from the working directory to the filesystem root, then down to the source.
  let climb: PathBuf = current
    .components()
    .skip(1)
    .map(|_| Path::new(".."))
    .collect::<PathBuf>()
    .join(
      source
        .strip_prefix(source.ancestors().last().expect("a root"))
        .expect("the source sits below its root"),
    );

  let mut sources: Vec<PathBuf> = vec![
    source.clone(),
    climb,
    source
      .parent()
      .expect("a directory")
      .join("..")
      .join("input")
      .join("st_test.json"),
  ];

  // Verbatim-prefixed, which a drive-qualified component would otherwise carry into the target. A verbatim path takes
  // no `/`, so it is rebuilt from components first.
  if cfg!(windows) {
    sources.push(PathBuf::from(format!(
      r"\\?\{}",
      source.components().collect::<PathBuf>().display()
    )));
  }

  for path in sources {
    if output.exists() {
      fs::remove_dir_all(&output)?;
    }

    assert_built_inside_output(&path, &output)?;
  }

  Ok(())
}

#[test]
fn a_source_in_a_directory_builds_into_the_language_directory() -> XrfResult {
  let test_root: &str = "build_targets/nested_source";
  let output_root = GeneratedResources::path(&format!("{test_root}/output"));

  if output_root.exists() {
    fs::remove_dir_all(&output_root)?;
  }

  GeneratedResources::write(&format!("{test_root}/source/ui/st_ui.json"), SOURCE_JSON)?;
  GeneratedResources::write(&format!("{test_root}/source/st_items.json"), SOURCE_JSON)?;

  let result = TranslationBuilder::build_roots(
    &roots(&format!("{test_root}/source")),
    None,
    &options(output_root.clone()),
  )?;

  assert!(output_root.join("eng/st_ui.xml").is_file());
  assert!(output_root.join("eng/st_items.xml").is_file());
  assert!(!output_root.join("eng/ui").exists());

  assert_eq!(result.sources, 2);
  assert_eq!(result.files, 2);
  assert_eq!(result.languages.len(), 1);
  assert_eq!(result.languages[0].language, "eng");
  assert_eq!(result.languages[0].files, 2);

  Ok(())
}

#[test]
fn two_sources_of_one_name_in_different_directories_are_refused() -> XrfResult {
  let test_root: &str = "build_targets/same_name";
  let output_root = GeneratedResources::path(&format!("{test_root}/output"));

  for relative_path in ["one/common.json", "two/common.json"] {
    GeneratedResources::write(&format!("{test_root}/source/{relative_path}"), SOURCE_JSON)?;
  }

  // The game loads tables by name alone, so the second would replace the first's strings or vanish under it.
  let error = TranslationBuilder::build_roots(
    &roots(&format!("{test_root}/source")),
    None,
    &options(output_root.clone()),
  )
  .unwrap_err();

  assert!(error.to_string().contains("both build to"), "{error}");
  assert!(!output_root.exists(), "nothing is written before the refusal");

  Ok(())
}

#[test]
fn builds_report_one_row_per_language() -> XrfResult {
  let test_root: &str = "build_targets/languages";
  let output_root = GeneratedResources::path(&format!("{test_root}/output"));

  if output_root.exists() {
    fs::remove_dir_all(&output_root)?;
  }

  GeneratedResources::write(&format!("{test_root}/source/st_a.json"), SOURCE_JSON)?;

  let mut all = options(output_root.clone());

  all.language = TranslationLanguage::All;

  let result = TranslationBuilder::build_roots(&roots(&format!("{test_root}/source")), None, &all)?;

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
fn builds_reject_two_sources_that_differ_only_in_case() -> XrfResult {
  // Checked against the target rule directly, because a case-insensitive host cannot hold both to be walked.
  let error = TranslationBuildTargets::validate(
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
  let source_root = GeneratedResources::path(&format!("{test_root}/source"));
  let other_root = GeneratedResources::path(&format!("{test_root}/other"));

  GeneratedResources::write(&format!("{test_root}/source/common.json"), SOURCE_JSON)?;
  GeneratedResources::write(&format!("{test_root}/other/common.json"), SOURCE_JSON)?;

  let layered: XrayRoots = XrayRoots::new([
    xrf_vfs::XrayRoot::new(other_root.clone(), XrayMountMode::Directory),
    xrf_vfs::XrayRoot::new(source_root.clone(), XrayMountMode::Directory),
  ]);

  let error = TranslationBuildTargets::ensure_output_outside_roots(&layered, &source_root.join("built")).unwrap_err();

  assert!(error.to_string().contains("must be outside source directory"));

  Ok(())
}
