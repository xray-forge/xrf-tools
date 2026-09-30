//! The environment check: `xrf-environment`'s findings, placed in the report, and the textures its keyframes name.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::time::Instant;

use xrf_engine_target::XrayEngine;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentFinding, EnvironmentReadOptions, EnvironmentReader, EnvironmentRule,
  EnvironmentSection, WeatherDescriptor, WeatherKey,
};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayLogicalPath;

use super::verify_environment_result::GamedataEnvironmentVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  /// Verifies every environment config as this project's engine reads it, and that each texture its weather keyframes
  /// name is in the game.
  ///
  /// # Errors
  ///
  /// Returns an error on cancellation, or when the configs cannot be listed.
  pub fn verify_environment(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataEnvironmentVerificationResult> {
    options.job.check_cancelled()?;

    xrf_output::heading!(options.output, "Verify environment:");

    let started_at: Instant = Instant::now();
    let read: EnvironmentReadOptions = EnvironmentReadOptions::default()
      .with_engine(self.engine)
      .with_job(options.job.clone());
    let catalog: EnvironmentCatalog = EnvironmentReader::read_opt(&self.ltx_project, &read)?;

    let mut findings: Vec<Finding> = catalog
      .findings
      .iter()
      .map(|finding| self.report_environment_finding(finding))
      .collect::<XrfResult<_>>()?;
    let mut invalid: BTreeSet<&str> = catalog.findings.iter().map(|finding| finding.file.as_str()).collect();

    for (file, finding) in self.verify_environment_textures(&catalog, options)? {
      invalid.insert(file);
      findings.push(finding);
    }

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    for finding in &findings {
      xrf_output::error!(
        options.output,
        "{}: {}",
        finding.message(),
        finding.subject().unwrap_or_default()
      );
    }

    let configs: BTreeSet<&str> = catalog.configs.iter().map(String::as_str).collect();
    let checked_configs_count: u32 = u32::try_from(configs.union(&invalid).count())
      .map_err(|_| XrfError::new_verify_error("Environment config count exceeds the supported result range"))?;
    let invalid_configs_count: u32 = u32::try_from(invalid.len())
      .map_err(|_| XrfError::new_verify_error("Environment config count exceeds the supported result range"))?;
    let duration = started_at.elapsed();

    xrf_output::info!(
      options.output,
      "Verified environment configs as {} in {}, {}/{} valid",
      self.engine,
      xrf_utils::format_duration(duration),
      checked_configs_count - invalid_configs_count,
      checked_configs_count
    );

    Ok(GamedataEnvironmentVerificationResult {
      checked_configs_count,
      duration,
      findings,
      invalid_configs_count,
    })
  }

  /// Every sky cube, its `#small` twin and every clouds texture a keyframe names, each looked up once; each finding
  /// with the config it is in.
  fn verify_environment_textures<'c>(
    &self,
    catalog: &'c EnvironmentCatalog,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<Vec<(&'c str, Finding)>> {
    let mut is_present: HashMap<String, bool> = HashMap::new();
    let mut findings: Vec<(&'c str, Finding)> = Vec::new();

    for cycle in catalog.cycles.iter().chain(&catalog.effects) {
      options.job.check_cancelled()?;

      let reported: PathBuf = self.ltx_project.path_of(&XrayLogicalPath::new(&cycle.file)?);

      for keyframe in &cycle.keyframes {
        for (what, texture) in Self::list_keyframe_textures(&keyframe.section, catalog.engine) {
          let is_found: bool = *is_present
            .entry(texture.clone())
            .or_insert_with(|| self.has_texture(&texture));

          if !is_found {
            let message: String = format!(
              "{} [{}] references missing {what} [{texture}]",
              cycle.kind.get_subject(),
              keyframe.section.name
            );

            findings.push((
              cycle.file.as_str(),
              GamedataFindingFactory::for_asset(GamedataVerificationRule::EnvironmentAsset, &reported, message),
            ));
          }
        }
      }
    }

    Ok(findings)
  }

  /// The textures a keyframe draws, by what they are: its sky cube and the cube's `#small` twin, and its clouds. An
  /// empty name draws nothing rather than a missing texture, as Anomaly's cycles name no clouds.
  fn list_keyframe_textures(
    section: &EnvironmentSection<WeatherKey>,
    engine: XrayEngine,
  ) -> Vec<(&'static str, String)> {
    let sky: &str = section.get_text(WeatherKey::SkyTexture, engine);
    let clouds: &str = section.get_text(WeatherKey::CloudsTexture, engine);
    let mut textures: Vec<(&'static str, String)> = Vec::new();

    if !sky.is_empty() {
      textures.push(("sky texture", sky.to_owned()));
      textures.push(("sky texture", format!("{sky}{}", WeatherDescriptor::ENVIRONMENT_SUFFIX)));
    }

    if !clouds.is_empty() {
      textures.push(("clouds texture", clouds.to_owned()));
    }

    textures
  }

  /// Whether the game holds a texture, as the engine finds one.
  fn has_texture(&self, texture: &str) -> bool {
    self
      .vfs()
      .scoped(self.scope())
      .dds_texture(texture)
      .ok()
      .flatten()
      .is_some()
  }

  fn report_environment_finding(&self, finding: &EnvironmentFinding) -> XrfResult<Finding> {
    let rule: GamedataVerificationRule = match finding.rule {
      EnvironmentRule::Engine => GamedataVerificationRule::EnvironmentEngine,
      EnvironmentRule::Reference => GamedataVerificationRule::EnvironmentReference,
      EnvironmentRule::Convention => GamedataVerificationRule::EnvironmentConvention,
    };
    let reported: PathBuf = self.ltx_project.path_of(&XrayLogicalPath::new(&finding.file)?);

    Ok(GamedataFindingFactory::for_asset(
      rule,
      reported,
      finding.message.clone(),
    ))
  }
}
