use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_error::XrfError;

use crate::project::animations::animations_verifier::AnimationsVerifier;
use crate::project::collisions::collisions_verifier::CollisionsVerifier;
use crate::project::coverage::coverage_verifier::CoverageVerifier;
use crate::project::environment::environment_verifier::EnvironmentVerifier;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::levels::levels_verifier::LevelsVerifier;
use crate::project::ltx::ltx_verifier::LtxVerifier;
use crate::project::materials::materials_verifier::MaterialsVerifier;
use crate::project::meshes::meshes_verifier::MeshesVerifier;
use crate::project::particles::particles_usage_verifier::ParticlesUsageVerifier;
use crate::project::particles::particles_verifier::ParticlesVerifier;
use crate::project::scripts::scripts_verifier::ScriptsVerifier;
use crate::project::shaders::shaders_verifier::ShadersVerifier;
use crate::project::sounds::sounds_verifier::SoundsVerifier;
use crate::project::spawns::spawns_verifier::SpawnsVerifier;
use crate::project::textures::textures_verifier::TexturesVerifier;
use crate::project::weapons::weapons_verifier::WeaponsVerifier;
use crate::{GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationCheckReport};

/// One check a verification runs, spelled on the wire and the command line as it displays.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Display, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamedataVerificationType {
  /// Files one source holds but the engine cannot reach, rather than any kind of asset authored inside them.
  #[display("collisions")]
  Collisions,
  /// Declared sources the run could not open, rather than anything authored inside the ones it did.
  #[display("coverage")]
  Coverage,
  #[display("animations")]
  Animations,
  /// Everything under `configs\environment` the engine reads, and the textures its keyframes name.
  #[display("environment")]
  Environment,
  #[display("levels")]
  Levels,
  #[display("ltx")]
  Ltx,
  /// The game material library, and every name the engine, configs and spawn objects resolve against it.
  #[display("materials")]
  Materials,
  #[display("meshes")]
  Meshes,
  #[display("particles")]
  Particles,
  #[display("particles-usage")]
  ParticlesUsage,
  #[display("scripts")]
  Scripts,
  #[display("shaders")]
  Shaders,
  #[display("sounds")]
  Sounds,
  #[display("spawns")]
  Spawns,
  #[display("textures")]
  Textures,
  #[display("weapons")]
  Weapons,
}

impl GamedataVerificationType {
  /// Every check a caller can select, which is every kind of authored asset.
  pub const ALL: [Self; 14] = [
    Self::Animations,
    Self::Environment,
    Self::Levels,
    Self::Ltx,
    Self::Materials,
    Self::Meshes,
    Self::Particles,
    Self::ParticlesUsage,
    Self::Scripts,
    Self::Shaders,
    Self::Sounds,
    Self::Spawns,
    Self::Textures,
    Self::Weapons,
  ];

  pub fn get_all() -> Vec<GamedataVerificationType> {
    Self::ALL.to_vec()
  }

  /// Runs this check over a project and reports it, a check that could not be performed included.
  pub fn run(
    self,
    project: &GamedataProject,
    options: &GamedataProjectVerifyOptions,
  ) -> GamedataVerificationCheckReport {
    match self {
      Self::Collisions => self.run_check::<CollisionsVerifier>(project, options),
      Self::Coverage => self.run_check::<CoverageVerifier>(project, options),
      Self::Animations => self.run_check::<AnimationsVerifier>(project, options),
      Self::Environment => self.run_check::<EnvironmentVerifier>(project, options),
      Self::Levels => self.run_check::<LevelsVerifier>(project, options),
      Self::Ltx => self.run_check::<LtxVerifier>(project, options),
      Self::Materials => self.run_check::<MaterialsVerifier>(project, options),
      Self::Meshes => self.run_check::<MeshesVerifier>(project, options),
      Self::Particles => self.run_check::<ParticlesVerifier>(project, options),
      Self::ParticlesUsage => self.run_check::<ParticlesUsageVerifier>(project, options),
      Self::Scripts => self.run_check::<ScriptsVerifier>(project, options),
      Self::Shaders => self.run_check::<ShadersVerifier>(project, options),
      Self::Sounds => self.run_check::<SoundsVerifier>(project, options),
      Self::Spawns => self.run_check::<SpawnsVerifier>(project, options),
      Self::Textures => self.run_check::<TexturesVerifier>(project, options),
      Self::Weapons => self.run_check::<WeaponsVerifier>(project, options),
    }
  }

  fn run_check<'a, C>(
    self,
    project: &'a GamedataProject,
    options: &'a GamedataProjectVerifyOptions,
  ) -> GamedataVerificationCheckReport
  where
    C: GamedataCheck<'a>,
  {
    GamedataVerificationCheckReport::from_check_result(self, C::new(project, options).verify())
  }
}

impl FromStr for GamedataVerificationType {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    Self::ALL
      .into_iter()
      .find(|verification_type| verification_type.to_string() == string)
      .ok_or_else(|| {
        XrfError::new_unexpected_error(format!(
          "Unexpected verification type '{verification}' provided",
          verification = string
        ))
      })
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataVerificationType;

  #[test]
  fn serializes_as_it_displays() {
    for verification_type in GamedataVerificationType::ALL {
      assert_eq!(
        serde_json::to_value(verification_type).expect("serializes"),
        serde_json::Value::String(verification_type.to_string())
      );
    }
  }

  #[test]
  fn parses_every_registered_verification_type() {
    for verification_type in GamedataVerificationType::ALL {
      let parsed: GamedataVerificationType = verification_type
        .to_string()
        .parse()
        .expect("Expected verification type to parse");

      assert_eq!(parsed, verification_type);
    }
  }

  /// An always-run check is not a selection, so naming it is a usage error rather than a redundant request.
  #[test]
  fn refuses_the_always_run_checks_as_a_selection() {
    for always_run in [GamedataVerificationType::Collisions, GamedataVerificationType::Coverage] {
      assert!(!GamedataVerificationType::ALL.contains(&always_run));
      assert!(always_run.to_string().parse::<GamedataVerificationType>().is_err());
    }
  }
}
