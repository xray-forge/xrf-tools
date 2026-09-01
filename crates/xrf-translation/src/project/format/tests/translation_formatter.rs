use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;
use xrf_utils::LineEndings;

use crate::project::format::translation_format_options::TranslationFormatOptions;
use crate::project::format::translation_format_result::TranslationFormatResult;
use crate::project::format::translation_formatter::TranslationFormatter;

fn endings(line_endings: LineEndings) -> TranslationFormatOptions {
  TranslationFormatOptions::default().with_line_endings(Some(line_endings))
}

fn format(path: &Path) -> XrfResult<TranslationFormatResult> {
  TranslationFormatter::format(std::slice::from_ref(&path.to_path_buf()))
}

fn check(path: &Path) -> XrfResult<TranslationFormatResult> {
  TranslationFormatter::check_format(std::slice::from_ref(&path.to_path_buf()))
}

fn read(path: &Path) -> String {
  String::from_utf8(fs::read(path).expect("Expected a readable source")).expect("Expected UTF-8")
}

#[test]
fn sorts_ids_naturally_rather_than_by_byte() -> XrfResult {
  let path: PathBuf = write_generated_test_resource(
    "translation_format/natural.json",
    "{\n  \"st_a10\": {\n    \"eng\": \"A\"\n  },\n  \"st_a2\": {\n    \"eng\": \"B\"\n  }\n}\n",
  )?;

  let result: TranslationFormatResult = format(&path)?;

  assert_eq!(result.invalid_files, 1);
  assert_eq!(result.to_format, vec![path.clone()]);
  // Byte order would leave `st_a10` first, which is the whole reason this is not `Ord for String`.
  assert_eq!(
    read(&path),
    "{\n  \"st_a2\": {\n    \"eng\": \"B\"\n  },\n  \"st_a10\": {\n    \"eng\": \"A\"\n  }\n}\n"
  );

  Ok(())
}

#[test]
fn sorts_language_keys() -> XrfResult {
  let path: PathBuf = write_generated_test_resource(
    "translation_format/languages.json",
    "{\n  \"st_a\": {\n    \"ukr\": \"U\",\n    \"eng\": \"E\"\n  }\n}\n",
  )?;

  format(&path)?;

  assert_eq!(
    read(&path),
    "{\n  \"st_a\": {\n    \"eng\": \"E\",\n    \"ukr\": \"U\"\n  }\n}\n"
  );

  Ok(())
}

#[test]
fn adds_a_trailing_newline() -> XrfResult {
  // 31 of the 34 hand-authored engine sources are shaped exactly like this.
  let path: PathBuf = write_generated_test_resource(
    "translation_format/no_newline.json",
    "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}",
  )?;

  let result: TranslationFormatResult = format(&path)?;

  assert_eq!(result.invalid_files, 1);
  assert!(read(&path).ends_with("}\n"));

  Ok(())
}

#[test]
fn rewrites_indentation() -> XrfResult {
  let path: PathBuf = write_generated_test_resource("translation_format/indent.json", "{\"st_a\":{\"eng\":\"A\"}}\n")?;

  format(&path)?;

  assert_eq!(read(&path), "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n");

  Ok(())
}

#[test]
fn leaves_a_canonical_source_untouched() -> XrfResult {
  let path: PathBuf = write_generated_test_resource(
    "translation_format/canonical.json",
    "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n",
  )?;

  let before: std::time::SystemTime = fs::metadata(&path)?.modified()?;
  let result: TranslationFormatResult = format(&path)?;

  assert_eq!(result.invalid_files, 0);
  assert_eq!(result.valid_files, 1);
  assert_eq!(result.total_files, 1);
  assert!(result.to_format.is_empty());
  // Not replaced with itself: a run over a clean tree touches no mtimes and provokes no watcher.
  assert_eq!(fs::metadata(&path)?.modified()?, before);

  Ok(())
}

#[test]
fn preserves_the_line_endings_a_file_already_uses() -> XrfResult {
  // A CRLF checkout is the normal state of `xrf-engine`, and a formatter that flipped it would rewrite every file to
  // say nothing about their content.
  let path: PathBuf = write_generated_test_resource(
    "translation_format/crlf.json",
    "{\r\n  \"st_b\": {\r\n    \"eng\": \"B\"\r\n  },\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n",
  )?;

  format(&path)?;

  let formatted: String = read(&path);

  assert!(formatted.starts_with("{\r\n  \"st_a\""));
  assert!(!formatted.contains("\n\n"));
  assert_eq!(formatted.matches("\r\n").count(), formatted.matches('\n').count());

  Ok(())
}

#[test]
fn a_check_over_a_file_with_other_line_endings_passes_unless_endings_were_asked_for() -> XrfResult {
  let path: PathBuf = write_generated_test_resource(
    "translation_format/endings_check.json",
    "{\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n",
  )?;

  assert_eq!(check(&path)?.invalid_files, 0);
  assert_eq!(
    TranslationFormatter::check_format_opt(std::slice::from_ref(&path), endings(LineEndings::Crlf))?.invalid_files,
    0
  );
  // Asserting the other spelling arms the check on it.
  assert_eq!(
    TranslationFormatter::check_format_opt(std::slice::from_ref(&path), endings(LineEndings::Lf))?.invalid_files,
    1
  );

  Ok(())
}

#[test]
fn an_asserted_ending_is_written() -> XrfResult {
  let path: PathBuf = write_generated_test_resource(
    "translation_format/endings_write.json",
    "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n",
  )?;

  TranslationFormatter::format_opt(std::slice::from_ref(&path), endings(LineEndings::Crlf))?;

  assert_eq!(read(&path), "{\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n");

  Ok(())
}

#[test]
fn a_check_reports_without_writing() -> XrfResult {
  let source: &str = "{\n  \"st_b\": {\n    \"eng\": \"B\"\n  },\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n";
  let path: PathBuf = write_generated_test_resource("translation_format/check.json", source)?;

  let result: TranslationFormatResult = check(&path)?;

  assert_eq!(result.invalid_files, 1);
  assert_eq!(result.to_format, vec![path.clone()]);
  assert_eq!(read(&path), source);

  Ok(())
}

#[test]
fn the_value_shape_is_left_alone() -> XrfResult {
  // Collapsing a one-element array or splitting an embedded line break are lossless conversions `build` undoes either
  // way, and choosing between the spellings is the importer's call about authoring, not a formatter's.
  let source: &str = "{\n  \"st_a\": {\n    \"eng\": [\n      \"one\"\n    ],\n    \"rus\": null\n  }\n}\n";
  let path: PathBuf = write_generated_test_resource("translation_format/shapes.json", source)?;

  let result: TranslationFormatResult = format(&path)?;

  assert_eq!(result.invalid_files, 0);
  assert_eq!(read(&path), source);

  Ok(())
}

#[test]
fn a_malformed_source_stops_the_run() -> XrfResult {
  let broken: PathBuf = write_generated_test_resource("translation_format/broken/a_broken.json", "{ not json")?;
  let sound: PathBuf = write_generated_test_resource(
    "translation_format/broken/b_sound.json",
    "{\n  \"st_b\": {\n    \"eng\": \"B\"\n  },\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n",
  )?;

  let directory: &Path = broken.parent().expect("Expected a parent directory");
  let error = TranslationFormatter::format(std::slice::from_ref(&directory.to_path_buf()))
    .expect_err("Expected a malformed source to stop the run");

  assert!(error.to_string().contains("a_broken.json"), "{error}");
  // Selection is sorted, so the sound file comes after the broken one and is never reached.
  assert!(read(&sound).starts_with("{\n  \"st_b\""));

  Ok(())
}

#[test]
fn a_named_file_is_formatted_whatever_its_extension() -> XrfResult {
  let path: PathBuf = write_generated_test_resource(
    "translation_format/named.txt",
    "{\n  \"st_b\": {\n    \"eng\": \"B\"\n  },\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n",
  )?;

  assert_eq!(format(&path)?.invalid_files, 1);
  assert!(read(&path).starts_with("{\n  \"st_a\""));

  Ok(())
}
