use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use super::super::scope::resolve;
use crate::language::TranslationLanguage;
use crate::project::tests::{roots, table};

#[test]
fn an_installation_root_resolves_down_to_the_language_directory() -> XrfResult {
  // The common invocation: name the mod folder, name a language, and let the tool find the tables.
  let root: &str = "parse_scope/from_root";

  write_generated_test_resource(&format!("{root}/configs/text/eng/st_a.xml"), table("st_a", "A"))?;

  let scope = resolve(&roots(root).open()?, None, TranslationLanguage::English)?;

  assert_eq!(scope.prefix(), r"configs\text\eng");

  Ok(())
}

#[test]
fn a_text_root_resolves_down_to_the_language_directory() -> XrfResult {
  let root: &str = "parse_scope/from_text";

  write_generated_test_resource(&format!("{root}/eng/st_a.xml"), table("st_a", "A"))?;

  let scope = resolve(&roots(root).open()?, None, TranslationLanguage::English)?;

  assert_eq!(scope.prefix(), "eng");

  Ok(())
}

#[test]
fn a_language_directory_named_directly_is_taken_as_it_is() -> XrfResult {
  // Nothing to descend into, so the root itself is the scope.
  let root: &str = "parse_scope/leaf";

  write_generated_test_resource(&format!("{root}/st_a.xml"), table("st_a", "A"))?;

  let scope = resolve(&roots(root).open()?, None, TranslationLanguage::English)?;

  assert_eq!(scope.prefix(), "");
  assert_eq!(scope.describe(), "<root>");

  Ok(())
}

#[test]
fn an_explicit_prefix_is_obeyed_without_searching() -> XrfResult {
  let root: &str = "parse_scope/explicit";

  write_generated_test_resource(&format!("{root}/configs/text/rus/st_a.xml"), table("st_a", "А"))?;

  let scope = resolve(
    &roots(root).open()?,
    Some(r"configs\text\rus"),
    TranslationLanguage::Russian,
  )?;

  assert_eq!(scope.prefix(), r"configs\text\rus");

  Ok(())
}

#[test]
fn a_text_root_holding_several_languages_still_resolves_to_the_one_named() -> XrfResult {
  // Descending happens before the guard, so naming the text root of a two-language tree is not
  // ambiguous: the language directory says which files belong to the run.
  let root: &str = "parse_scope/several";

  write_generated_test_resource(&format!("{root}/eng/st_a.xml"), table("st_a", "A"))?;
  write_generated_test_resource(&format!("{root}/rus/st_a.xml"), table("st_a", "A"))?;

  let scope = resolve(&roots(root).open()?, None, TranslationLanguage::English)?;

  assert_eq!(scope.prefix(), "eng");

  Ok(())
}

#[test]
fn a_scope_still_holding_another_language_is_refused() -> XrfResult {
  // The dangerous shape: tables sit directly in the scope, so the run would read them as English, and
  // a Russian directory beside them would be swept up by the same recursive listing and filed as
  // English too. On an Anomaly-sized tree that is thousands of entries mislabelled with no complaint.
  let root: &str = "parse_scope/mixed";

  write_generated_test_resource(&format!("{root}/st_a.xml"), table("st_a", "A"))?;
  write_generated_test_resource(&format!("{root}/rus/st_b.xml"), table("st_b", "B"))?;

  let error = resolve(&roots(root).open()?, Some(""), TranslationLanguage::English).unwrap_err();

  assert!(error.to_string().contains("hold other languages"));
  assert!(error.to_string().contains("rus"));

  Ok(())
}

#[test]
fn map_desc_is_not_mistaken_for_another_language() -> XrfResult {
  // The engine excludes it by name rather than by content, so a tree carrying map descriptions is not
  // a tree carrying a second language.
  let root: &str = "parse_scope/map_desc";

  write_generated_test_resource(&format!("{root}/st_a.xml"), table("st_a", "A"))?;
  write_generated_test_resource(&format!("{root}/map_desc/st_b.xml"), table("st_b", "B"))?;

  let scope = resolve(&roots(root).open()?, Some(""), TranslationLanguage::English)?;

  assert_eq!(scope.prefix(), "");

  Ok(())
}
