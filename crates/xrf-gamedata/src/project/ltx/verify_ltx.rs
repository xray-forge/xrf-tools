use std::path::Path;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxFormatOptions, LtxProjectFormatResult, LtxProjectVerifyResult, LtxVerifyOptions};

use crate::GamedataFindingFactory;
use crate::project::ltx::verify_ltx_result::GamedataLtxVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  pub fn verify_ltx(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataLtxVerificationResult> {
    xrf_output::heading!(options.output, "Verify LTX files");

    let started_at: Instant = Instant::now();

    let format_result: LtxProjectFormatResult = self.verify_ltx_format(options)?;
    let verification_result: LtxProjectVerifyResult = self.verify_ltx_schemes(options)?;
    let findings: Vec<Finding> = Self::collect_ltx_findings(&format_result, &verification_result);

    let duration = started_at.elapsed();

    xrf_output::info!(
      options.output,
      "Verified gamedata ltx files in {}",
      xrf_utils::format_duration(duration),
    );

    Ok(GamedataLtxVerificationResult {
      duration,
      findings,
      format_result,
      verification_result,
    })
  }

  fn collect_ltx_findings(
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

    for error in &verification_result.errors {
      match error {
        XrfError::LtxScheme {
          at: Some(path),
          field,
          message,
          section,
        } => findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LtxSchema,
          Path::new(path),
          format!("[{section}] {field}: {message}"),
        )),
        error => findings.push(GamedataFindingFactory::without_asset(
          GamedataVerificationRule::LtxVerification,
          error.to_string(),
        )),
      }
    }

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    findings
  }

  fn verify_ltx_format(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<LtxProjectFormatResult> {
    xrf_output::heading!(options.output, "Verify LTX files formatting");

    // todo: Inert until gamedata verification is a job of its own. When it is, this is one of the levels
    // todo: it nests: the run counts its checks, and this check counts the files it reads.
    self.ltx_project.check_format_all_files_opt(LtxFormatOptions {
      output: options.output.clone(),
      ..Default::default()
    })
  }

  fn verify_ltx_schemes(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<LtxProjectVerifyResult> {
    xrf_output::heading!(options.output, "Verify LTX schemas");

    self.ltx_project.verify_entries_opt(LtxVerifyOptions {
      output: options.output.clone(),
      ..Default::default()
    })
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_error::XrfError;
  use xrf_ltx::{LtxProjectFormatResult, LtxProjectVerifyResult};

  use super::GamedataProject;
  use crate::GamedataFindingFactory;
  use crate::{Finding, GamedataVerificationRule};

  #[test]
  fn collects_format_and_scheme_findings_with_source_paths() {
    let format_result: LtxProjectFormatResult = LtxProjectFormatResult {
      to_format: vec![PathBuf::from("configs/system.ltx")],
      ..Default::default()
    };
    let verification_result: LtxProjectVerifyResult = LtxProjectVerifyResult {
      errors: vec![XrfError::new_scheme_error_at(
        "weather",
        "fog_density",
        "Expected a number",
        "configs/environment/weathers/test.ltx",
      )],
      ..Default::default()
    };

    let findings: Vec<Finding> = GamedataProject::collect_ltx_findings(&format_result, &verification_result);

    assert_eq!(
      findings,
      vec![
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
