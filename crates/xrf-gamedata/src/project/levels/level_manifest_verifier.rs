use xrf_ltx::Ltx;

use crate::GamedataFindingFactory;
use crate::project::levels::level_bundle::LevelBundle;
use crate::project::levels::level_engine_constants::{
  LEVEL_AI_FILE, LEVEL_DETAILS_FILE, LEVEL_DETAILS_TEXTURE_FILE, LEVEL_LTX_FILE, LEVEL_MAP_SECTION,
  LEVEL_MAP_TEXTURE_FIELD, REQUIRED_LEVEL_FILES,
};
use crate::project::levels::level_roster::RosterLevel;
use crate::{Finding, GamedataVerificationRule};

/// Verifies that a bundle contains the files a built level is made of, and that its own
/// configuration is readable and resolvable.
pub(crate) struct LevelManifestVerifier<'a> {
  bundle: &'a LevelBundle<'a>,
}

impl<'a> LevelManifestVerifier<'a> {
  pub(crate) fn new(bundle: &'a LevelBundle<'a>) -> Self {
    Self { bundle }
  }

  pub(crate) fn verify(&self, level: Option<&RosterLevel>) -> Vec<Finding> {
    let mut findings: Vec<Finding> = self.verify_required_files(level);

    findings.extend(self.verify_details_pair());
    findings.extend(self.verify_level_ltx());

    findings
  }

  fn verify_required_files(&self, level: Option<&RosterLevel>) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();
    let mut required: Vec<&str> = REQUIRED_LEVEL_FILES.to_vec();

    // AI-map requirement is derived from the game graph, never from the bundle name: a level the
    // graph can send the player to must be navigable.
    if level.is_some() {
      required.push(LEVEL_AI_FILE);
    }

    for file in required {
      match self.bundle.file_size(file) {
        None => findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsMissingFile,
          self.bundle.file_path(file),
          format!(
            "Level bundle [{}] does not contain required file [{file}]",
            self.bundle.name()
          ),
        )),
        Some(0) => findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsFileEmpty,
          self.bundle.file_path(file),
          format!(
            "Level bundle [{}] contains empty required file [{file}]",
            self.bundle.name()
          ),
        )),
        Some(_) => {}
      }
    }

    findings
  }

  /// Detail model description and its compiled texture atlas always ship together.
  fn verify_details_pair(&self) -> Vec<Finding> {
    let has_details: bool = self.bundle.contains(LEVEL_DETAILS_FILE);
    let has_details_texture: bool = self.bundle.contains(LEVEL_DETAILS_TEXTURE_FILE);

    if has_details == has_details_texture {
      return Vec::new();
    }

    let (present, missing) = if has_details {
      (LEVEL_DETAILS_FILE, LEVEL_DETAILS_TEXTURE_FILE)
    } else {
      (LEVEL_DETAILS_TEXTURE_FILE, LEVEL_DETAILS_FILE)
    };

    vec![GamedataFindingFactory::for_asset(
      GamedataVerificationRule::LevelsDetailsPair,
      self.bundle.file_path(missing),
      format!(
        "Level bundle [{}] contains [{present}] without its counterpart [{missing}]",
        self.bundle.name()
      ),
    )]
  }

  fn verify_level_ltx(&self) -> Vec<Finding> {
    let Some(path): Option<String> = self.bundle.resolved_file(LEVEL_LTX_FILE) else {
      // Absence is already reported by the required files rule.
      return Vec::new();
    };

    // Read through the project, so an archived level config is parsed rather than skipped and the same dialect
    // resolves it as resolves everything under `configs`. A level config is outside that prefix, which is why this
    // names a scope instead of a project path.
    let ltx: Ltx = match self
      .bundle
      .project()
      .ltx_project
      .read_full_in_scope(self.bundle.project().scope(), &path)
    {
      Ok(ltx) => ltx,
      Err(error) => {
        return vec![GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsLtxRead,
          self.bundle.file_path(LEVEL_LTX_FILE),
          format!("Failed to read level configuration: {error}"),
        )];
      }
    };

    let Some(texture) = ltx
      .section(LEVEL_MAP_SECTION)
      .and_then(|section| section.get(LEVEL_MAP_TEXTURE_FIELD))
    else {
      return Vec::new();
    };

    if self.bundle.resolves_texture(texture) {
      return Vec::new();
    }

    vec![GamedataFindingFactory::for_asset(
      GamedataVerificationRule::LevelsMapTexture,
      self.bundle.file_path(LEVEL_LTX_FILE),
      format!(
        "Level [{}] references missing level map texture [{texture}] in [{LEVEL_MAP_SECTION}]",
        self.bundle.name()
      ),
    )]
  }
}
