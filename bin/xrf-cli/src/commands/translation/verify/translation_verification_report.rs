use std::time::Duration;

use serde::Serialize;
use xrf_report::{CheckReport, Finding, Report};
use xrf_translation::{TranslationVerifyLanguageSummary, TranslationVerifyResult};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerificationReportOutput {
  checked_translations_count: u32,
  checks: Vec<TranslationVerificationCheckOutput>,
  /// One row per file and language, so "which languages are incomplete" is answerable without
  /// reading every finding - there are 149,979 of them for a two-language mod checked against eight.
  languages: Vec<TranslationVerificationLanguageOutput>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  missing_translations_count: u32,
  status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationVerificationLanguageOutput {
  checked: u32,
  file: String,
  language: String,
  missing: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationVerificationCheckOutput {
  #[serde(with = "xrf_utils::optional_duration_ms")]
  duration: Option<Duration>,
  findings: Vec<TranslationVerificationFindingOutput>,
  id: String,
  status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationVerificationFindingOutput {
  message: String,
  rule_id: String,
  subject: Option<String>,
}

/// Builds what a `translation verify` run reports to a machine.
pub struct TranslationVerificationReportPayload<'a> {
  result: &'a TranslationVerifyResult,
}

impl<'a> TranslationVerificationReportPayload<'a> {
  pub fn new(result: &'a TranslationVerifyResult) -> Self {
    Self { result }
  }

  pub fn build(&self) -> TranslationVerificationReportOutput {
    let report: Report = self.result.to_report();
    let checks: Vec<TranslationVerificationCheckOutput> = report.checks().iter().map(Self::check_output).collect();

    TranslationVerificationReportOutput {
      checked_translations_count: self.result.checked_translations_count,
      checks,
      languages: self.result.languages.iter().map(Self::language_output).collect(),
      duration: self.result.duration,
      missing_translations_count: self.result.missing_translations_count,
      status: report.status().to_string(),
    }
  }

  fn language_output(summary: &TranslationVerifyLanguageSummary) -> TranslationVerificationLanguageOutput {
    TranslationVerificationLanguageOutput {
      checked: summary.checked,
      file: summary.file.clone(),
      language: summary.language.clone(),
      missing: summary.missing,
    }
  }

  fn check_output(report: &CheckReport) -> TranslationVerificationCheckOutput {
    TranslationVerificationCheckOutput {
      duration: report.duration(),
      findings: report.findings().iter().map(Self::finding_output).collect(),
      id: report.id().to_string(),
      status: report.status().to_string(),
    }
  }

  fn finding_output(finding: &Finding) -> TranslationVerificationFindingOutput {
    TranslationVerificationFindingOutput {
      message: finding.message().to_string(),
      rule_id: finding.rule_id().to_string(),
      subject: finding.subject().map(String::from),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;
  use std::sync::atomic::{AtomicU64, Ordering};

  use xrf_translation::{TranslationLanguage, TranslationVerifier, TranslationVerifyOptions};

  use super::TranslationVerificationReportPayload;

  static NEXT_TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

  #[test]
  fn reports_missing_translations_as_structured_findings() {
    let unique: u64 = NEXT_TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = std::env::temp_dir().join(format!(
      "xrf-cli-translation-verification-report-test-{}-{unique}",
      std::process::id()
    ));
    let translation_path: PathBuf = root.join("dialogs.json");
    let options: TranslationVerifyOptions = TranslationVerifyOptions {
      is_strict: false,
      job: Default::default(),
      output: xrf_output::OutputOptions::default(),
      language: TranslationLanguage::Ukrainian,
      is_detailed: true,
    };

    fs::create_dir_all(&root).unwrap();
    fs::write(&translation_path, r#"{"st_dialog_hello":{"ukr":null}}"#).unwrap();

    let result = TranslationVerifier::verify_file(&translation_path, &options).unwrap();

    let json: serde_json::Value =
      serde_json::to_value(TranslationVerificationReportPayload::new(&result).build()).unwrap();

    fs::remove_dir_all(&root).unwrap();

    assert_eq!(json["status"], "failed");
    assert_eq!(json["checkedTranslationsCount"], 1);
    assert_eq!(json["missingTranslationsCount"], 1);
    assert_eq!(json["checks"][0]["id"], "translations");
    assert_eq!(json["checks"][0]["status"], "failed");
    assert_eq!(json["checks"][0]["findings"][0]["ruleId"], "translations.missing");
    assert_eq!(
      json["checks"][0]["findings"][0]["subject"],
      translation_path.to_string_lossy().replace('\\', "/")
    );
  }
}
