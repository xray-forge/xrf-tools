use serde::Serialize;
use xrf_error::XrfError;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_translation::{TranslationVerifyLanguageSummary, TranslationVerifyResult};

/// What a `translation verify` run reports to a machine: the counts, the per-language table and the shared report, whose
/// one check carries the run's duration.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerifyReport {
  checked_translations: u32,
  /// One row per file and language, so "which languages are incomplete" is answerable without reading every finding:
  /// there are 149,979 of them for a two-language mod checked against eight.
  languages: Vec<TranslationVerifyLanguageSummary>,
  missing_translations: u32,
  #[serde(flatten)]
  report: Report,
}

impl TranslationVerifyReport {
  pub fn new(result: &TranslationVerifyResult) -> Self {
    Self {
      checked_translations: result.checked_translations_count,
      languages: result.languages.clone(),
      missing_translations: result.missing_translations_count,
      report: result.to_report(),
    }
  }

  /// A source that could not be read as translations, judged as one finding: nothing in it was checked.
  pub fn malformed(error: &XrfError) -> Self {
    Self {
      checked_translations: 0,
      languages: Vec::new(),
      missing_translations: 0,
      report: Report::new(vec![CheckReport::new(
        CheckId::of_static("translations"),
        Status::Failed,
        None,
        vec![Finding::new(
          RuleId::of_static("translations.source"),
          None,
          error.to_string(),
        )],
      )]),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_error::XrfError;
  use xrf_test_utils::utils::create_generated_test_resource_dir;
  use xrf_translation::{TranslationLanguage, TranslationVerifier, TranslationVerifyOptions};

  use super::TranslationVerifyReport;

  #[test]
  fn reports_missing_translations_as_structured_findings() {
    let root: PathBuf = create_generated_test_resource_dir("translation_verify/report").unwrap();
    let translation_path: PathBuf = root.join("dialogs.json");
    let options: TranslationVerifyOptions = TranslationVerifyOptions {
      is_strict: false,
      job: Default::default(),
      output: xrf_output::OutputOptions::default(),
      language: TranslationLanguage::Ukrainian,
      is_detailed: true,
    };

    fs::write(&translation_path, r#"{"st_dialog_hello":{"ukr":null}}"#).unwrap();

    let result = TranslationVerifier::verify_file(&translation_path, &options).unwrap();
    let json: serde_json::Value = serde_json::to_value(TranslationVerifyReport::new(&result)).unwrap();

    fs::remove_dir_all(&root).unwrap();

    assert_eq!(json["status"], "failed");
    assert_eq!(json["checkedTranslations"], 1);
    assert_eq!(json["missingTranslations"], 1);
    assert_eq!(json["languages"][0]["missing"], 1);
    assert_eq!(json["checks"][0]["id"], "translations");
    assert_eq!(json["checks"][0]["status"], "failed");
    assert_eq!(json["checks"][0]["findings"][0]["ruleId"], "translations.missing");
    assert_eq!(
      json["checks"][0]["findings"][0]["subject"],
      xrf_utils::to_portable_path_string(&translation_path)
    );
  }

  /// A malformed source is judged content, so it reaches the report as a finding rather than leaving none.
  #[test]
  fn reports_a_malformed_source_as_one_finding() {
    let json: serde_json::Value = serde_json::to_value(TranslationVerifyReport::malformed(
      &XrfError::new_parsing_error("Expected a JSON object"),
    ))
    .unwrap();

    assert_eq!(json["status"], "failed");
    assert_eq!(json["checkedTranslations"], 0);
    assert_eq!(json["checks"][0]["findings"][0]["ruleId"], "translations.source");
    assert_eq!(json["checks"][0]["findings"][0]["subject"], serde_json::Value::Null);
  }
}
