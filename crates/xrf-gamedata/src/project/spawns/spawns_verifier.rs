use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_spawn::{SpawnFile, XRayByteOrder};

use crate::project::gamedata_check::GamedataCheck;
use crate::project::spawns::verify_spawns_result::GamedataSpawnsVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// The spawns check: every merged spawn file under `spawns` reads, the level spawn files aside.
pub(crate) struct SpawnsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for SpawnsVerifier<'a> {
  type Result = GamedataSpawnsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataSpawnsVerificationResult> {
    self.options.job.check_cancelled()?;

    let started_at: Instant = Instant::now();
    let spawn_paths: Vec<String> = self.project.spawn_paths();

    if spawn_paths.is_empty() {
      return Ok(GamedataSpawnsVerificationResult {
        duration: started_at.elapsed(),
        findings: Vec::new(),
        total_spawns: 0,
        invalid_spawns: 0,
      });
    }

    let mut total_spawns: u32 = 0;
    let mut findings: Vec<Finding> = Vec::new();
    let mut invalid_spawns: u32 = 0;

    for path in &spawn_paths {
      self.options.job.check_cancelled()?;

      total_spawns += 1;

      if let Some(finding) = self.verify_spawn(path) {
        findings.push(finding);
        invalid_spawns += 1;
      }
    }

    let duration: Duration = started_at.elapsed();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    self.options.job.check_cancelled()?;

    Ok(GamedataSpawnsVerificationResult {
      duration,
      findings,
      total_spawns,
      invalid_spawns,
    })
  }
}

impl SpawnsVerifier<'_> {
  /// The finding reading one spawn file raises, addressed by its logical path and read through the VFS, so an archived
  /// spawn file is verified too.
  fn verify_spawn(&self, path: &str) -> Option<Finding> {
    let error = self
      .project
      .read_parsed(path, SpawnFile::read_from_chunk::<XRayByteOrder, _>)
      .err()?;

    Some(GamedataFindingFactory::for_asset(
      GamedataVerificationRule::SpawnsRead,
      path,
      format!("Failed to read spawn file: {error}"),
    ))
  }
}
