use std::path::Path;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_ltx::{
  LtxFinding, LtxFindingKind, LtxFormatOptions, LtxProject, LtxProjectFormatResult, LtxProjectVerifyResult,
  LtxVerifyOptions,
};

use crate::project::gamedata_check::GamedataCheck;
use crate::project::ltx::verify_ltx_result::GamedataLtxVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// The LTX check: how every config is formatted, and whether it resolves against its section schemes.
pub(crate) struct LtxVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for LtxVerifier<'a> {
  type Result = GamedataLtxVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataLtxVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify LTX files");

    let started_at: Instant = Instant::now();

    let format_result: LtxProjectFormatResult = self.verify_format()?;
    let verification_result: LtxProjectVerifyResult = self.verify_schemes()?;
    let findings: Vec<Finding> =
      Self::collect_findings(&self.project.ltx_project, &format_result, &verification_result);

    let duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata ltx files in {}",
      xrf_utils::format_duration(duration),
    );

    self.options.job.check_cancelled()?;

    Ok(GamedataLtxVerificationResult {
      duration,
      findings,
      format_result,
      verification_result,
    })
  }
}

impl LtxVerifier<'_> {
  fn collect_findings(
    ltx_project: &LtxProject,
    format_result: &LtxProjectFormatResult,
    verification_result: &LtxProjectVerifyResult,
  ) -> Vec<Finding> {
    let mut findings: Vec<Finding> = format_result
      .to_format
      .iter()
      .map(|path| {
        GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LtxFormatting,
          path,
          "LTX file needs formatting",
        )
      })
      .collect();

    for finding in &verification_result.findings {
      // The declaring config where one is known and the entry point that resolved it otherwise, so a finding always
      // carries the asset a person opens.
      let reported: LtxFinding = ltx_project.with_reported_paths(finding);

      findings.push(GamedataFindingFactory::for_asset(
        Self::get_rule(finding.kind),
        Path::new(reported.file.as_deref().unwrap_or(&reported.entry)),
        finding.get_located_message(),
      ));
    }

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    findings
  }

  fn get_rule(kind: LtxFindingKind) -> GamedataVerificationRule {
    match kind {
      LtxFindingKind::Scheme => GamedataVerificationRule::LtxSchema,
      LtxFindingKind::Dialect => GamedataVerificationRule::LtxDialect,
      LtxFindingKind::Parse | LtxFindingKind::Include | LtxFindingKind::RootKeys | LtxFindingKind::Unreadable => {
        GamedataVerificationRule::LtxVerification
      }
    }
  }

  fn verify_format(&self) -> XrfResult<LtxProjectFormatResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify LTX files formatting");

    self.project.ltx_project.check_format_all_files_opt(LtxFormatOptions {
      output: self.options.output.clone(),
      job: self.options.job.clone(),
    })
  }

  fn verify_schemes(&self) -> XrfResult<LtxProjectVerifyResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify LTX schemas");

    self.project.ltx_project.verify_entries_opt(LtxVerifyOptions {
      output: self.options.output.clone(),
      job: self.options.job.clone(),
    })
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_ltx::{LtxFinding, LtxProject, LtxProjectFormatResult, LtxProjectVerifyResult};
  use xrf_test_utils::GeneratedResources;

  use super::LtxVerifier;
  use crate::{Finding, GamedataFindingFactory, GamedataVerificationRule};

  #[test]
  fn collects_format_scheme_and_unreadable_findings_with_source_paths() {
    let root: PathBuf = GeneratedResources::create_dir("gamedata_ltx_findings/sources").expect("project root");
    let project: LtxProject = LtxProject::open_at_path(&root).expect("an empty project opens");
    let format_result: LtxProjectFormatResult = LtxProjectFormatResult {
      to_format: vec![PathBuf::from("configs/system.ltx")],
      ..Default::default()
    };
    let verification_result: LtxProjectVerifyResult = LtxProjectVerifyResult {
      findings: vec![
        LtxFinding::new_scheme(
          "configs/system.ltx",
          Some(String::from("configs/environment/weathers/test.ltx")),
          "weather",
          Some("fog_density"),
          "Expected a number",
        ),
        LtxFinding::new_unreadable("configs/broken.ltx", "Ltx parse error: 1:1 broken"),
      ],
      ..Default::default()
    };

    let findings: Vec<Finding> = LtxVerifier::collect_findings(&project, &format_result, &verification_result);

    assert_eq!(
      findings,
      vec![
        GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LtxVerification,
          "configs/broken.ltx",
          "Ltx parse error: 1:1 broken",
        ),
        GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LtxSchema,
          "configs/environment/weathers/test.ltx",
          "[weather] fog_density: Expected a number",
        ),
        GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LtxFormatting,
          "configs/system.ltx",
          "LTX file needs formatting",
        ),
      ]
    );
  }
}
