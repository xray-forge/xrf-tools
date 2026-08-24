use std::path::Path;
use std::time::{Duration, Instant};

use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::levels::level_engine_constants::SPAWNS_DIRECTORY;
use crate::project::spawns::verify_spawns_result::GamedataSpawnsVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  /// Verify spawn files in spawns directories, not levels spawn files.
  pub fn verify_spawns(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataSpawnsVerificationResult> {
    let started_at: Instant = Instant::now();

    let spawn_files: Vec<String> = self
      .entries_of_type(AssetType::Spawn)
      .into_iter()
      .filter(|location| location.get_logical_path().is_under(SPAWNS_DIRECTORY).unwrap_or(false))
      .map(|location| location.get_logical_path().to_string())
      .collect();

    xrf_output::heading!(options.output, "{} {}", "Verify spawns:", spawn_files.len());

    if spawn_files.is_empty() {
      xrf_output::info!(options.output, "No spawn files found in gamedata root");

      // todo: Verify result struct.

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

    for relative_path in &spawn_files {
      total_spawns += 1;

      // Read through the VFS, so an archived spawn file is verified rather than reported missing.
      if self.find(relative_path).ok().flatten().is_some() {
        let spawn_findings: Vec<Finding> = self.verify_spawn_findings(options, relative_path);

        if !spawn_findings.is_empty() {
          findings.extend(spawn_findings);
          invalid_spawns += 1;
        }
      } else {
        findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::SpawnsPath,
          Path::new(relative_path),
          "Spawn path was not found in gamedata roots",
        ));
        invalid_spawns += 1;
      }
    }

    let duration: Duration = started_at.elapsed();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    xrf_output::info!(
      options.output,
      "Verified gamedata spawn files in {}, {}/{} are valid",
      xrf_utils::format_duration(duration),
      total_spawns - invalid_spawns,
      total_spawns
    );

    Ok(GamedataSpawnsVerificationResult {
      duration,
      findings,
      total_spawns,
      invalid_spawns,
    })
  }

  /// Findings from reading one spawn file, addressed by its logical path.
  fn verify_spawn_findings(&self, options: &GamedataProjectVerifyOptions, path: &str) -> Vec<Finding> {
    let file_path: String = path.to_string();

    xrf_output::verbose!(options.output, "Verify spawn file: {}", file_path);

    match self.read_parsed(AssetType::Spawn, path, |chunk| {
      SpawnFile::read_from_chunk::<XRayByteOrder, _>(chunk)
    }) {
      Ok(_) => {
        xrf_output::verbose!(options.output, "Verify spawn file: {}", file_path);

        Vec::new()
      }
      Err(error) => {
        xrf_output::error!(
          options.output,
          "Spawn file validation failed: {} -> {}",
          file_path,
          error
        );

        vec![GamedataFindingFactory::for_asset(
          GamedataVerificationRule::SpawnsRead,
          path,
          format!("Failed to read spawn file: {error}"),
        )]
      }
    }
  }
}
