use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use crate::project::format::translation_source_selection::select_sources;

const SOURCE: &str = "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n";

#[test]
fn walks_a_directory_for_json_sources_only() -> XrfResult {
  let source: PathBuf = write_generated_test_resource("translation_selection/walk/st_a.json", SOURCE)?;
  let nested: PathBuf = write_generated_test_resource("translation_selection/walk/nested/st_b.json", SOURCE)?;

  write_generated_test_resource("translation_selection/walk/st_a.eng.xml", "<string_table/>")?;
  write_generated_test_resource("translation_selection/walk/notes.txt", "ignored")?;

  let directory: &Path = source.parent().expect("Expected a parent directory");
  let selected: Vec<PathBuf> = select_sources(&[directory.to_path_buf()])?;

  assert_eq!(selected, vec![nested, source]);

  Ok(())
}

#[test]
fn takes_a_named_file_whatever_its_extension() -> XrfResult {
  let path: PathBuf = write_generated_test_resource("translation_selection/named/st_a.txt", SOURCE)?;

  assert_eq!(select_sources(std::slice::from_ref(&path))?, vec![path]);

  Ok(())
}

#[test]
fn de_duplicates_overlapping_paths() -> XrfResult {
  let path: PathBuf = write_generated_test_resource("translation_selection/overlap/st_a.json", SOURCE)?;
  let directory: &Path = path.parent().expect("Expected a parent directory");

  assert_eq!(
    select_sources(&[directory.to_path_buf(), path.clone()])?,
    vec![path.clone()]
  );
  assert_eq!(select_sources(&[path.clone(), path.clone()])?, vec![path]);

  Ok(())
}

#[test]
fn a_path_that_does_not_exist_is_refused() {
  let error = select_sources(&[PathBuf::from("no_such_translation_directory")])
    .expect_err("Expected a missing path to be refused");

  assert!(error.to_string().contains("does not exist"), "{error}");
}

#[test]
fn selecting_nothing_is_refused() -> XrfResult {
  // Not `ltx format`'s empty-set success: a renamed directory would otherwise make a check gate pass over no files at
  // all, which reads identically to a clean tree.
  let placeholder: PathBuf = write_generated_test_resource("translation_selection/empty/notes.txt", "ignored")?;
  let directory: &Path = placeholder.parent().expect("Expected a parent directory");

  let error = select_sources(&[directory.to_path_buf()]).expect_err("Expected an empty selection to be refused");

  assert!(
    error.to_string().contains("No translation sources to format"),
    "{error}"
  );

  Ok(())
}
