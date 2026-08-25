use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use super::roots;
use crate::project::descriptor::TranslationProjectMode;
use crate::project::layout::detect_mode;

/// Each layout is looked for under its own prefix, so a fixture has to sit where the layout keeps it.
/// That is the contract: the caller names a root to mount, not the directory the tables sit in.
#[test]
fn language_directories_under_the_text_prefix_look_like_gamedata() -> XrfResult {
  let root: &str = "layout/gamedata";

  write_generated_test_resource(
    &format!("{root}/configs/text/rus/st_test.xml"),
    "<string_table></string_table>",
  )?;

  assert_eq!(detect_mode(&roots(root))?, TranslationProjectMode::Gamedata);

  Ok(())
}

#[test]
fn a_json_map_looks_like_a_source_tree() -> XrfResult {
  let root: &str = "layout/source_json";

  write_generated_test_resource(
    &format!("{root}/translations/st_test.json"),
    r#"{"st_hello":{"eng":"Hello"}}"#,
  )?;

  assert_eq!(detect_mode(&roots(root))?, TranslationProjectMode::Source);

  Ok(())
}

#[test]
fn a_language_suffixed_xml_looks_like_a_source_tree() -> XrfResult {
  let root: &str = "layout/source_xml";

  write_generated_test_resource(
    &format!("{root}/translations/dialogs.eng.xml"),
    "<string_table></string_table>",
  )?;

  assert_eq!(detect_mode(&roots(root))?, TranslationProjectMode::Source);

  Ok(())
}

#[test]
fn a_tree_holding_both_reads_as_a_source_tree() -> XrfResult {
  // Only a source tree has the files that decide it, so its evidence outranks a language directory
  // the same checkout also ships.
  let root: &str = "layout/both";

  write_generated_test_resource(
    &format!("{root}/translations/st_test.json"),
    r#"{"st_hello":{"eng":"Hello"}}"#,
  )?;
  write_generated_test_resource(
    &format!("{root}/configs/text/rus/st_test.xml"),
    "<string_table></string_table>",
  )?;

  assert_eq!(detect_mode(&roots(root))?, TranslationProjectMode::Source);

  Ok(())
}

#[test]
fn an_unreadable_root_falls_back_to_source() -> XrfResult {
  // Guessing gamedata for something unreadable would preselect the mode that rewrites shipped files.
  assert_eq!(
    detect_mode(&roots("layout/does_not_exist"))?,
    TranslationProjectMode::Source
  );

  Ok(())
}
