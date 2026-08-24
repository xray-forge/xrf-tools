use std::path::Path;

use serde::Serialize;
use xrf_report::{CheckReport, Finding, Report};
use xrf_translation::ProjectVerifyResult;

use crate::core::generic_command::CommandResult;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationVerificationReportOutput {
  checked_translations_count: u32,
  checks: Vec<TranslationVerificationCheckOutput>,
  duration_ms: u64,
  missing_translations_count: u32,
  status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationVerificationCheckOutput {
  duration_ms: Option<u64>,
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

pub struct TranslationVerificationReportWriter<'a> {
  result: &'a ProjectVerifyResult,
}

impl<'a> TranslationVerificationReportWriter<'a> {
  pub fn new(result: &'a ProjectVerifyResult) -> Self {
    Self { result }
  }

  pub fn write(&self, report_path: &Path) -> CommandResult {
    let output: TranslationVerificationReportOutput = self.report_output();
    let json: String = serde_json::to_string_pretty(&output)?;

    std::fs::write(report_path, format!("{json}\n"))?;

    Ok(())
  }

  fn report_output(&self) -> TranslationVerificationReportOutput {
    let report: Report = self.result.to_report();
    let checks: Vec<TranslationVerificationCheckOutput> = report.checks().iter().map(Self::check_output).collect();

    TranslationVerificationReportOutput {
      checked_translations_count: self.result.checked_translations_count,
      checks,
      duration_ms: xrf_utils::duration_to_millis(self.result.duration),
      missing_translations_count: self.result.missing_translations_count,
      status: report.status().to_string(),
    }
  }

  fn check_output(report: &CheckReport) -> TranslationVerificationCheckOutput {
    TranslationVerificationCheckOutput {
      duration_ms: report.duration().map(xrf_utils::duration_to_millis),
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

  use xrf_translation::{ProjectVerifyOptions, TranslationLanguage, verify_file};

  use super::TranslationVerificationReportWriter;

  static NEXT_TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

  #[test]
  fn writes_missing_translations_as_structured_findings() {
    let unique: u64 = NEXT_TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = std::env::temp_dir().join(format!(
      "xrf-cli-translation-verification-report-test-{}-{unique}",
      std::process::id()
    ));
    let translation_path: PathBuf = root.join("dialogs.json");
    let report_path: PathBuf = root.join("report.json");
    let options: ProjectVerifyOptions = ProjectVerifyOptions {
      is_strict: false,
      output: xrf_output::OutputOptions::default(),
      language: TranslationLanguage::Ukrainian,
      path: root.clone(),
    };

    fs::create_dir_all(&root).unwrap();
    fs::write(&translation_path, r#"{"st_dialog_hello":{"ukr":null}}"#).unwrap();

    let result = verify_file(&translation_path, &options).unwrap();

    TranslationVerificationReportWriter::new(&result)
      .write(&report_path)
      .unwrap();
    let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();

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
