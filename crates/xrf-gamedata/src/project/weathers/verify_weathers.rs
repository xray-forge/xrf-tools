//! Discovery and aggregate reporting for assembled weather cycles.

use std::collections::BTreeSet;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayLogicalPath;

use super::verify_weathers_result::GamedataWeathersVerificationResult;
use super::weather_definitions::WeatherDefinitions;
use super::weather_validator::verify_weather_findings_with_definitions;
use crate::GamedataFindingFactory;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Logical directory holding assembled weather cycles.
///
/// Spelled with X-Ray separators rather than joined with `Path`, because the project addresses its configs logically; a host
/// join would read `environment/weathers` off Windows and match nothing.
const WEATHERS_DIRECTORY: &str = "environment\\weathers";

impl GamedataProject {
  /// Verifies every assembled weather cycle under `configs/environment/weathers`.
  ///
  /// Weather definitions are loaded once and reused across all discovered cycle files. A missing
  /// cycle directory, an empty cycle directory, or any invalid cycle produces a failed result.
  pub fn verify_weathers(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataWeathersVerificationResult> {
    options.job.check_cancelled()?;

    xrf_output::heading!(options.output, "Verify weathers:");

    let started_at: Instant = Instant::now();

    let weather_configs: Vec<&XrayLogicalPath> = self
      .ltx_project
      .ltx_files
      .iter()
      .filter(|path| {
        path
          .parent()
          .is_some_and(|parent| parent.as_str().ends_with(WEATHERS_DIRECTORY))
      })
      .collect();

    let checked_weather_files_count: u32 = u32::try_from(weather_configs.len())
      .map_err(|_| XrfError::new_verify_error("Weather config count exceeds the supported result range"))?;
    let definitions: WeatherDefinitions = WeatherDefinitions::read(&self.ltx_project);
    let mut definition_load_errors: BTreeSet<String> = BTreeSet::new();
    let mut findings: Vec<Finding> = Vec::new();
    let mut invalid_weather_files_count: u32 = 0;

    for weather_config in weather_configs {
      options.job.check_cancelled()?;

      let weather_findings: Vec<Finding> = verify_weather_findings_with_definitions(
        self,
        options,
        weather_config,
        &definitions,
        &mut definition_load_errors,
      )?;

      if !weather_findings.is_empty() {
        findings.extend(weather_findings);
        invalid_weather_files_count += 1;
      }
    }

    if checked_weather_files_count == 0 {
      findings.push(GamedataFindingFactory::without_asset(
        GamedataVerificationRule::WeathersFiles,
        "No weather files found",
      ));
    }

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    let duration = started_at.elapsed();

    for error in definition_load_errors {
      options.job.check_cancelled()?;

      options.output.error(error);
    }

    if checked_weather_files_count == 0 {
      xrf_output::info!(
        options.output,
        "Checked gamedata weather files in {}, no weather files found",
        xrf_utils::format_duration(duration)
      );
    } else {
      xrf_output::info!(
        options.output,
        "Verified gamedata weather files in {}, {}/{} valid",
        xrf_utils::format_duration(duration),
        checked_weather_files_count - invalid_weather_files_count,
        checked_weather_files_count
      );
    }

    options.job.check_cancelled()?;

    Ok(GamedataWeathersVerificationResult {
      duration,
      checked_weather_files_count,
      findings,
      invalid_weather_files_count,
    })
  }
}
