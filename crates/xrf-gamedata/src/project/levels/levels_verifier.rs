use std::sync::Arc;

use std::collections::BTreeSet;
use std::time::Instant;

use xrf_db::ShaderLibraryFile;
use xrf_error::XrfResult;

use crate::GamedataFindingFactory;
use crate::project::levels::level_binaries_verifier::{LevelBinariesOutcome, LevelBinariesVerifier};
use crate::project::levels::level_bundle::LevelBundle;
use crate::project::levels::level_manifest_verifier::LevelManifestVerifier;
use crate::project::levels::level_reconciliation_verifier::LevelReconciliationVerifier;
use crate::project::levels::level_references_verifier::{LevelReferencesOutcome, LevelReferencesVerifier};
use crate::project::levels::level_roster::LevelRoster;
use crate::project::levels::verify_levels_result::GamedataLevelsVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions};

pub(crate) struct LevelsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> LevelsVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataLevelsVerificationResult> {
    xrf_output::heading!(self.options.output, "Verify levels:");

    let started_at: Instant = Instant::now();
    let mut roster: LevelRoster = LevelRoster::read(self.project)?;

    if !roster.has_source() {
      xrf_output::info!(
        self.options.output,
        "No spawn files carrying a game graph found in gamedata root, level roster is unknown"
      );

      return Ok(GamedataLevelsVerificationResult::skipped(started_at.elapsed()));
    }

    let reconciliation: LevelReconciliationVerifier = LevelReconciliationVerifier::new(self.project, self.options);
    let bundles: BTreeSet<String> = reconciliation.bundle_names()?;

    let mut findings: Vec<Finding> = std::mem::take(&mut roster.findings);

    findings.extend(reconciliation.verify(&roster, &bundles)?);

    let shader_library: Option<Arc<ShaderLibraryFile>> = LevelReferencesVerifier::read_library(self.project);

    let mut checked_references_count: u32 = 0;
    let mut invalid_references_count: u32 = 0;
    let mut invalid_levels_count: u32 = 0;

    for name in &bundles {
      xrf_output::verbose!(self.options.output, "Verify level bundle: {name}");

      let bundle: LevelBundle = LevelBundle::new(self.project, name);
      let level = roster.find(name);

      let mut bundle_findings: Vec<Finding> = LevelManifestVerifier::new(&bundle).verify(level);
      let binaries: LevelBinariesOutcome = LevelBinariesVerifier::new(&bundle).verify(level);

      bundle_findings.extend(binaries.findings);

      if let Some(shaders) = &binaries.shaders {
        let references: LevelReferencesOutcome =
          LevelReferencesVerifier::new(&bundle, shader_library.as_deref()).verify(shaders);

        checked_references_count += references.checked_count;
        invalid_references_count += references.invalid_count;
        bundle_findings.extend(references.findings);
      }

      // todo: Detail model closure belongs here. `level.details` holds the detail model list, whose
      //   entries are OGF references resolvable through `assets.ogf()`. It is deferred because the
      //   `CDetailManager` slot grid is the only level format in scope that needs real research, and
      //   25 of 94 inspected bundles ship no `level.details` at all. Cost is not the blocker: the
      //   assembled tree holds 22 MB of detail data, well inside the check budget.

      // todo: Static sound closure belongs here. `level.snd_static` holds level sound sources that
      //   should resolve against the sound index the same way `verify_sounds` resolves references.
      //   It is deferred because it needs a new format reader and only 39 of 94 inspected bundles
      //   ship one.

      if !bundle_findings.is_empty() {
        invalid_levels_count += 1;
        findings.extend(bundle_findings);
      }
    }

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    let result: GamedataLevelsVerificationResult = GamedataLevelsVerificationResult {
      duration: started_at.elapsed(),
      findings,
      has_roster: true,
      roster_levels_count: roster.levels.len() as u32,
      checked_levels_count: bundles.len() as u32,
      invalid_levels_count,
      checked_references_count,
      invalid_references_count,
    };

    xrf_output::info!(
      self.options.output,
      "Verified gamedata levels in {}, {} graph levels, {}/{} level bundles valid, {}/{} shader references valid",
      xrf_utils::format_duration(result.duration),
      result.roster_levels_count,
      result.checked_levels_count - result.invalid_levels_count,
      result.checked_levels_count,
      result.checked_references_count - result.invalid_references_count,
      result.checked_references_count,
    );

    Ok(result)
  }
}
