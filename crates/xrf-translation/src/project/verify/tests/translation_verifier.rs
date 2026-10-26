use xrf_error::XrfResult;
use xrf_report::Status;
use xrf_test_utils::GeneratedResources;

use crate::language::translation_language::TranslationLanguage;
use crate::project::tests::fixtures::roots;
use crate::project::verify::translation_verifier::TranslationVerifier;
use crate::project::verify::translation_verify_options::TranslationVerifyOptions;
use crate::project::verify::translation_verify_result::TranslationVerifyResult;
use crate::xml::translation_line_limit::TranslationLineLimit;

fn options(language: TranslationLanguage) -> TranslationVerifyOptions {
  TranslationVerifyOptions {
    job: Default::default(),
    is_strict: false,
    output: xrf_output::OutputOptions::default(),
    language,
    is_detailed: true,
  }
}

fn rules(result: &TranslationVerifyResult) -> Vec<&str> {
  result.problems.iter().map(|problem| problem.rule.as_str()).collect()
}

#[test]
fn a_complete_source_the_game_loads_passes_both_checks() -> XrfResult {
  let root: &str = "verify/clean";

  GeneratedResources::write(&format!("{root}/st_a.json"), r#"{"st_a":{"eng":"A","rus":"А"}}"#)?;

  let result = TranslationVerifier::verify_roots(&roots(root), None, &options(TranslationLanguage::English))?;

  assert!(result.problems.is_empty());
  assert_eq!(result.status(), Status::Passed);

  Ok(())
}

#[test]
fn reports_a_character_the_language_cannot_hold_and_a_line_the_game_refuses() -> XrfResult {
  let root: &str = "verify/engine";
  let long: String = "a".repeat(TranslationLineLimit::MAX_BYTES);

  GeneratedResources::write(
    &format!("{root}/st_a.json"),
    format!(r#"{{"st_accent":{{"eng":"José"}},"st_long":{{"eng":"{long}"}}}}"#),
  )?;

  let result = TranslationVerifier::verify_roots(&roots(root), None, &options(TranslationLanguage::English))?;

  assert_eq!(
    rules(&result),
    vec!["translations.unencodable", "translations.overlong"]
  );
  // Nothing is missing, and the run fails anyway: these are tables the game shows wrong or will not load.
  assert_eq!(result.missing_translations_count, 0);
  assert_eq!(result.get_engine_status(), Status::Failed);
  assert_eq!(result.to_engine_report().status(), Status::Failed);

  Ok(())
}

#[test]
fn reports_what_only_the_sources_together_make_a_problem() -> XrfResult {
  let root: &str = "verify/across";

  GeneratedResources::write(&format!("{root}/one/common.json"), r#"{"st_same":{"eng":"one"}}"#)?;
  GeneratedResources::write(&format!("{root}/two/common.json"), r#"{"st_same":{"eng":"two"}}"#)?;

  let result = TranslationVerifier::verify_roots(&roots(root), None, &options(TranslationLanguage::English))?;

  assert_eq!(
    rules(&result),
    vec!["translations.duplicate-across-files", "translations.collision"]
  );

  Ok(())
}

#[test]
fn a_missing_translation_fails_only_the_completeness_check() -> XrfResult {
  let root: &str = "verify/incomplete";

  GeneratedResources::write(&format!("{root}/st_a.json"), r#"{"st_a":{"eng":"A","ukr":null}}"#)?;

  let result = TranslationVerifier::verify_roots(&roots(root), None, &options(TranslationLanguage::Ukrainian))?;
  let report = result.to_report();

  assert_eq!(result.missing_translations_count, 1);
  assert_eq!(result.get_engine_status(), Status::Passed);
  assert_eq!(
    report
      .checks()
      .iter()
      .map(|check| (check.id().as_str(), check.status()))
      .collect::<Vec<_>>(),
    vec![
      (TranslationVerifyResult::COMPLETENESS_CHECK, Status::Failed),
      (TranslationVerifyResult::ENGINE_CHECK, Status::Passed)
    ]
  );

  Ok(())
}
