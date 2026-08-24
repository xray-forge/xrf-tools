use std::sync::Arc;

use xrf_db::{
  LevelAiFile, LevelAiHeader, LevelCformFile, LevelCformHeader, LevelFile, LevelShadersChunk, XRayByteOrder,
};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::levels::level_bundle::LevelBundle;
use crate::project::levels::level_engine_constants::{
  AI_CURRENT_VERSION, AI_VERSION_ALLOWED, CFORM_CURRENT_VERSION, LEVEL_AI_FILE, LEVEL_CFORM_FILE, LEVEL_FILE,
  LEVEL_PRODUCTION_VERSION,
};
use crate::project::levels::level_roster::RosterLevel;
use crate::{Finding, GamedataVerificationRule};

/// Findings from the binary level files, plus the shader table left for reference closure.
#[derive(Default)]
pub(crate) struct LevelBinariesOutcome {
  pub(crate) findings: Vec<Finding>,
  pub(crate) shaders: Option<LevelShadersChunk>,
}

/// Reproduces the engine assertions raised while loading a level bundle.
///
/// Every rule here corresponds to an `R_ASSERT` in the engine, so a finding means the game refuses
/// to load the level rather than that the data is merely unusual.
pub(crate) struct LevelBinariesVerifier<'a> {
  bundle: &'a LevelBundle<'a>,
}

impl<'a> LevelBinariesVerifier<'a> {
  pub(crate) fn new(bundle: &'a LevelBundle<'a>) -> Self {
    Self { bundle }
  }

  pub(crate) fn verify(&self, level: Option<&RosterLevel>) -> LevelBinariesOutcome {
    let mut outcome: LevelBinariesOutcome = self.verify_level_file();

    outcome.findings.extend(self.verify_cform());
    outcome.findings.extend(self.verify_ai_map(level));

    outcome
  }

  /// `R_ASSERT2(XRCL_PRODUCTION_VERSION == H.XRLC_version, "Incompatible level version.")` and
  /// `R_ASSERT2(chunk, "Level doesn't builded correctly.")`.
  fn verify_level_file(&self) -> LevelBinariesOutcome {
    let Some(path): Option<String> = self.bundle.resolved_file(LEVEL_FILE) else {
      return LevelBinariesOutcome::default();
    };

    let asset_path: String = self.bundle.file_path(LEVEL_FILE);

    let level_file: Arc<LevelFile> = match self.bundle.project().read_parsed(AssetType::Level, &path, |chunk| {
      LevelFile::read_from_chunk::<XRayByteOrder, _>(chunk)
    }) {
      Ok(level_file) => level_file,
      Err(error) => {
        return LevelBinariesOutcome {
          findings: vec![GamedataFindingFactory::for_asset(
            GamedataVerificationRule::LevelsFileTruncated,
            &asset_path,
            format!("Failed to read level file chunks: {error}"),
          )],
          shaders: None,
        };
      }
    };

    let mut findings: Vec<Finding> = Vec::new();

    if level_file.header.xrlc_version != LEVEL_PRODUCTION_VERSION {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsHeaderVersion,
        &asset_path,
        format!(
          "Level is compiled with version {} but the engine loads only version {LEVEL_PRODUCTION_VERSION}",
          level_file.header.xrlc_version
        ),
      ));
    }

    if level_file.shaders.is_none() {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsShadersChunk,
        &asset_path,
        String::from("Level file has no shaders chunk, the renderer asserts with 'Level doesn't builded correctly.'"),
      ));
    }

    LevelBinariesOutcome {
      findings,
      shaders: level_file.shaders.clone(),
    }
  }

  /// `R_ASSERT(CFORM_CURRENT_VERSION == H.version)`.
  fn verify_cform(&self) -> Vec<Finding> {
    let Some(path): Option<String> = self.bundle.resolved_file(LEVEL_CFORM_FILE) else {
      return Vec::new();
    };

    if self.bundle.file_size(LEVEL_CFORM_FILE) < Some(LevelCformHeader::SIZE) {
      return vec![GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsFileTruncated,
        self.bundle.file_path(LEVEL_CFORM_FILE),
        format!(
          "Level collision form is smaller than its {} byte header",
          LevelCformHeader::SIZE
        ),
      )];
    }

    let cform: Arc<LevelCformFile> = match self.bundle.project().read_parsed(AssetType::CForm, &path, |chunks| {
      LevelCformFile::read_from_chunk::<XRayByteOrder, _>(chunks)
    }) {
      Ok(cform) => cform,
      Err(error) => {
        return vec![GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsFileTruncated,
          self.bundle.file_path(LEVEL_CFORM_FILE),
          format!("Failed to read level collision form header: {error}"),
        )];
      }
    };

    if cform.header.version == CFORM_CURRENT_VERSION {
      return Vec::new();
    }

    vec![GamedataFindingFactory::for_asset(
      GamedataVerificationRule::LevelsCformVersion,
      self.bundle.file_path(LEVEL_CFORM_FILE),
      format!(
        "Level collision form has version {} but the engine loads only version {CFORM_CURRENT_VERSION}",
        cform.header.version
      ),
    )]
  }

  /// `ASSERT_XRAI_VERSION_MATCH` plus the three guid assertions raised by `AISpaceBase::Load`.
  fn verify_ai_map(&self, level: Option<&RosterLevel>) -> Vec<Finding> {
    let Some(path): Option<String> = self.bundle.resolved_file(LEVEL_AI_FILE) else {
      return Vec::new();
    };

    if self.bundle.file_size(LEVEL_AI_FILE) < Some(LevelAiHeader::SIZE) {
      return vec![GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsFileTruncated,
        self.bundle.file_path(LEVEL_AI_FILE),
        format!("Level AI-map is smaller than its {} byte header", LevelAiHeader::SIZE),
      )];
    }

    let ai: Arc<LevelAiFile> = match self.bundle.project().read_parsed(AssetType::Ai, &path, |chunks| {
      LevelAiFile::read_from_chunk::<XRayByteOrder, _>(chunks)
    }) {
      Ok(ai) => ai,
      Err(error) => {
        return vec![GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsFileTruncated,
          self.bundle.file_path(LEVEL_AI_FILE),
          format!("Failed to read level AI-map header: {error}"),
        )];
      }
    };

    let mut findings: Vec<Finding> = Vec::new();
    let asset_path: String = self.bundle.file_path(LEVEL_AI_FILE);

    if ai.header.version < AI_VERSION_ALLOWED || ai.header.version > AI_CURRENT_VERSION {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsAiVersion,
        &asset_path,
        format!(
          "Level AI-map has version {} but the engine loads only versions {AI_VERSION_ALLOWED} to {AI_CURRENT_VERSION}",
          ai.header.version
        ),
      ));
    }

    // Remaining rules compare the AI-map against the game graph, so they only apply to levels the
    // graph actually declares.
    let Some(level) = level else {
      return findings;
    };

    if level.guid != ai.header.guid {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsLevelGuid,
        &asset_path,
        format!(
          "Game graph declares level [{}] with guid {} but its AI-map has guid {}, the engine asserts with 'graph doesn't correspond to the AI-map'",
          level.name, level.guid, ai.header.guid
        ),
      ));
    }

    if let Some(cross_table_level_guid) = level.cross_table_level_guid
      && cross_table_level_guid != ai.header.guid
    {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsAiGuid,
        &asset_path,
        format!(
          "Cross table of level [{}] has level guid {cross_table_level_guid} but its AI-map has guid {}, the engine asserts with 'cross_table doesn't correspond to the AI-map'",
          level.name, ai.header.guid
        ),
      ));
    }

    if let Some(cross_table_game_guid) = level.cross_table_game_guid
      && cross_table_game_guid != level.graph_guid
    {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsGraphGuid,
        &asset_path,
        format!(
          "Cross table of level [{}] has game guid {cross_table_game_guid} but the game graph has guid {}, the engine asserts with 'graph doesn't correspond to the cross table'",
          level.name, level.graph_guid
        ),
      ));
    }

    if let Some(cross_table_nodes_count) = level.cross_table_nodes_count
      && cross_table_nodes_count != ai.header.count
    {
      findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsAiNodeCount,
        &asset_path,
        format!(
          "Cross table of level [{}] describes {cross_table_nodes_count} AI nodes but its AI-map contains {}, release builds do not assert on this and silently corrupt navigation",
          level.name, ai.header.count
        ),
      ));
    }

    findings
  }
}
