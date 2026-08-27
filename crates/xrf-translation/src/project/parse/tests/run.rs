use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};
use xrf_utils::encode_string_to_bytes;

use super::super::options::ProjectParseOptions;
use super::super::result::ProjectParseResult;
use super::super::run::parse_translations;
use crate::language::TranslationLanguage;
use crate::project::tests::{roots, table};

fn options(root: &str, language: TranslationLanguage, output_dir: PathBuf) -> ProjectParseOptions {
  ProjectParseOptions {
    output: xrf_output::OutputOptions::default(),
    roots: roots(root),
    prefix: None,
    language,
    output_dir,
    file: None,
    is_overwrite: false,
    is_dry_run: false,
  }
}

/// A clean output directory below the generated test tree.
fn output(test_root: &str) -> XrfResult<PathBuf> {
  let path: PathBuf = build_absolute_generated_test_resource_path(&format!("{test_root}/output"));

  if path.exists() {
    fs::remove_dir_all(&path)?;
  }

  Ok(path)
}

fn read(path: &PathBuf) -> String {
  fs::read_to_string(path).expect("Expected the written source")
}

/// Write a string table the way a shipped one is written: declared, and in its language's code page.
///
/// Not UTF-8. A gamedata table carries no byte order mark, so a Russian one written as UTF-8 decodes
/// as windows-1251 and comes back as mojibake — correctly, since that is what the file claims to be.
fn write_table(relative_path: &str, language: TranslationLanguage, id: &str, text: &str) -> XrfResult {
  let source: String = format!(
    "<?xml version=\"1.0\" encoding=\"{}\" ?>
{}",
    language.get_language_encoding(),
    table(id, text)
  );

  write_generated_test_resource(
    relative_path,
    encode_string_to_bytes(&source, language.new_language_encoder())?,
  )?;

  Ok(())
}

#[test]
fn imports_one_language_into_a_new_source() -> XrfResult {
  let root: &str = "parse_run/first_import";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_medkit", "Medkit"))?;

  let result: ProjectParseResult =
    parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  assert_eq!(result.census.files_read, 1);
  assert_eq!(result.census.files_created, 1);
  assert_eq!(result.census.entries_inserted, 1);
  assert_eq!(result.language, "eng");
  assert!(!result.is_dry_run);

  let written: String = read(&output_dir.join("st_items.json"));

  assert!(written.contains("\"st_medkit\""));
  assert!(written.contains("\"eng\": \"Medkit\""));
  // New files end with a newline, which is what a diff and every other tool expects.
  assert!(written.ends_with('\n'));

  Ok(())
}

#[test]
fn a_second_language_merges_into_the_same_file() -> XrfResult {
  let root: &str = "parse_run/second_language";
  let output_dir: PathBuf = output(root)?;

  write_table(
    &format!("{root}/eng/st_items.xml"),
    TranslationLanguage::English,
    "st_medkit",
    "Medkit",
  )?;
  write_table(
    &format!("{root}/rus/st_items.xml"),
    TranslationLanguage::Russian,
    "st_bread",
    "Хлеб",
  )?;

  parse_translations(&options(
    &format!("{root}/eng"),
    TranslationLanguage::English,
    output_dir.clone(),
  ))?;

  let second: ProjectParseResult = parse_translations(&options(
    &format!("{root}/rus"),
    TranslationLanguage::Russian,
    output_dir.clone(),
  ))?;

  assert_eq!(second.census.files_updated, 1);
  assert_eq!(second.census.entries_inserted, 1);
  // `st_medkit` has no Russian and `st_bread` no English, so both gaps become visible.
  assert_eq!(second.census.placeholders_added, 2);

  let merged: serde_json::Value = serde_json::from_str(&read(&output_dir.join("st_items.json"))).unwrap();

  assert_eq!(merged["st_medkit"]["eng"], "Medkit");
  assert!(merged["st_medkit"]["rus"].is_null());
  assert_eq!(merged["st_bread"]["rus"], "Хлеб");
  assert!(merged["st_bread"]["eng"].is_null());

  Ok(())
}

#[test]
fn running_the_same_import_twice_writes_nothing_the_second_time() -> XrfResult {
  let root: &str = "parse_run/idempotent";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_medkit", "Medkit"))?;

  parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  let again: ProjectParseResult = parse_translations(&options(root, TranslationLanguage::English, output_dir))?;

  assert_eq!(again.census.files_unchanged, 1);
  assert_eq!(again.census.files_updated, 0);
  assert_eq!(again.census.entries_unchanged, 1);

  Ok(())
}

#[test]
fn existing_text_survives_unless_overwrite_is_asked_for() -> XrfResult {
  let root: &str = "parse_run/conflict";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_medkit", "Medkit"))?;

  fs::create_dir_all(&output_dir)?;
  fs::write(
    output_dir.join("st_items.json"),
    r#"{"st_medkit":{"eng":"First aid kit"}}"#,
  )?;

  let kept: ProjectParseResult = parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  assert_eq!(kept.census.entries_conflicted, 1);
  assert!(read(&output_dir.join("st_items.json")).contains("First aid kit"));

  let mut overwriting = options(root, TranslationLanguage::English, output_dir.clone());

  overwriting.is_overwrite = true;

  let replaced: ProjectParseResult = parse_translations(&overwriting)?;

  assert_eq!(replaced.census.entries_conflicted, 1);
  assert!(read(&output_dir.join("st_items.json")).contains("Medkit"));

  Ok(())
}

#[test]
fn a_dry_run_reports_what_it_would_have_written_and_writes_nothing() -> XrfResult {
  let root: &str = "parse_run/dry";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_medkit", "Medkit"))?;

  let mut dry = options(root, TranslationLanguage::English, output_dir.clone());

  dry.is_dry_run = true;

  let result: ProjectParseResult = parse_translations(&dry)?;

  assert!(result.is_dry_run);
  assert_eq!(result.census.files_created, 1);
  assert_eq!(result.census.entries_inserted, 1);
  assert!(!output_dir.join("st_items.json").exists());

  Ok(())
}

#[test]
fn nested_sources_keep_their_directories_apart() -> XrfResult {
  // Two tables of the same name in different directories must not merge into one file.
  let root: &str = "parse_run/nested";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/one/common.xml"), table("st_one", "One"))?;
  write_generated_test_resource(&format!("{root}/eng/two/common.xml"), table("st_two", "Two"))?;

  parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  assert!(output_dir.join("one/common.json").is_file());
  assert!(output_dir.join("two/common.json").is_file());

  Ok(())
}

#[test]
fn a_single_table_can_be_selected_by_name() -> XrfResult {
  let root: &str = "parse_run/selected";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_medkit", "Medkit"))?;
  write_generated_test_resource(&format!("{root}/eng/st_ui.xml"), table("st_start", "Start"))?;

  let mut selected = options(root, TranslationLanguage::English, output_dir.clone());

  selected.file = Some(String::from("st_items.xml"));

  let result: ProjectParseResult = parse_translations(&selected)?;

  assert_eq!(result.census.files_read, 1);
  assert!(output_dir.join("st_items.json").is_file());
  assert!(!output_dir.join("st_ui.json").exists());

  Ok(())
}

#[test]
fn an_unreadable_table_costs_its_own_strings_and_not_the_import() -> XrfResult {
  let root: &str = "parse_run/damaged";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_good.xml"), table("st_medkit", "Medkit"))?;
  write_generated_test_resource(&format!("{root}/eng/st_bad.xml"), "<string_table><string id=")?;

  let result: ProjectParseResult =
    parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  assert_eq!(result.census.files_read, 1);
  assert_eq!(result.census.files_skipped, 1);
  assert!(output_dir.join("st_good.json").is_file());
  assert!(
    result
      .get_findings()
      .iter()
      .any(|finding| finding.rule_id().as_str() == "translations.unreadable")
  );

  Ok(())
}

#[test]
fn xml_that_is_not_a_string_table_is_skipped_rather_than_written_empty() -> XrfResult {
  // OpenXRay's marker file sits in language directories and is not a table. Writing `{}` for it would
  // put a file into the source tree that means nothing.
  let root: &str = "parse_run/not_a_table";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/openxray.xml"), "<openxray></openxray>")?;

  let result: ProjectParseResult =
    parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  assert_eq!(result.census.files_read, 0);
  assert_eq!(result.census.files_skipped, 1);
  assert!(!output_dir.join("openxray.json").exists());

  Ok(())
}

#[test]
fn a_repeated_id_keeps_the_last_and_says_so() -> XrfResult {
  let root: &str = "parse_run/duplicate";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(
    &format!("{root}/eng/st_items.xml"),
    "<string_table>\
       <string id=\"st_a\"><text>first</text></string>\
       <string id=\"st_a\"><text>second</text></string>\
     </string_table>",
  )?;

  let result: ProjectParseResult =
    parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  // What `CStringTable::Load` leaves in the table is the last one.
  assert!(read(&output_dir.join("st_items.json")).contains("second"));
  assert!(
    result
      .get_findings()
      .iter()
      .any(|finding| finding.rule_id().as_str() == "translations.duplicate")
  );

  Ok(())
}

#[test]
fn multi_line_text_is_imported_as_an_array() -> XrfResult {
  let root: &str = "parse_run/multiline";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_a", "first\\nsecond"))?;

  parse_translations(&options(root, TranslationLanguage::English, output_dir.clone()))?;

  let written: serde_json::Value = serde_json::from_str(&read(&output_dir.join("st_items.json"))).unwrap();

  assert_eq!(written["st_a"]["eng"][0], "first");
  assert_eq!(written["st_a"]["eng"][1], "second");

  Ok(())
}

#[test]
fn a_scope_holding_no_string_tables_is_refused() -> XrfResult {
  // Exit 0 with an empty census is indistinguishable from a clean import, and a command that
  // silently succeeds is the defect this one was reported for. A mistyped prefix lands here.
  let root: &str = "parse_run/empty";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/notes.txt"), "not a translation")?;

  let error = parse_translations(&options(root, TranslationLanguage::English, output_dir)).unwrap_err();

  assert!(error.to_string().contains("No string tables to import"));

  Ok(())
}

#[test]
fn a_file_selector_that_names_nothing_is_refused() -> XrfResult {
  let root: &str = "parse_run/absent_file";
  let output_dir: PathBuf = output(root)?;

  write_generated_test_resource(&format!("{root}/eng/st_items.xml"), table("st_a", "A"))?;

  let mut selected = options(root, TranslationLanguage::English, output_dir);

  selected.file = Some(String::from("st_absent.xml"));

  let error = parse_translations(&selected).unwrap_err();

  assert!(error.to_string().contains("st_absent.xml"));

  Ok(())
}
