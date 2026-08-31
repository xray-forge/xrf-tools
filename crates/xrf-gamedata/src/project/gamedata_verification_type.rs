use std::str::FromStr;

use derive_more::Display;
use xrf_error::XrfError;

use crate::{GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationCheckReport};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Display)]
pub enum GamedataVerificationType {
  /// Files one source holds but the engine cannot reach, rather than any kind of asset authored inside them.
  ///
  /// Outside [`Self::ALL`], so `--checks` neither offers nor parses it: an unreachable file is missing
  /// from the game whichever kinds a caller asked about, and [`GamedataProject::verify`] runs this on every run.
  #[display("collisions")]
  Collisions,
  /// Declared sources the run could not open, rather than anything authored inside the ones it did.
  #[display("coverage")]
  Coverage,
  #[display("animations")]
  Animations,
  #[display("levels")]
  Levels,
  #[display("ltx")]
  Ltx,
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
  #[display("weathers")]
  Weathers,
}

impl GamedataVerificationType {
  /// Every check a caller can select, which is every kind of authored asset.
  ///
  /// [`Self::Collisions`] and [`Self::Coverage`] are absent on purpose; they judge the project's own inputs
  /// rather than a kind, and always run.
  pub const ALL: [Self; 13] = [
    Self::Animations,
    Self::Levels,
    Self::Ltx,
    Self::Meshes,
    Self::Particles,
    Self::ParticlesUsage,
    Self::Scripts,
    Self::Shaders,
    Self::Sounds,
    Self::Spawns,
    Self::Textures,
    Self::Weapons,
    Self::Weathers,
  ];

  pub fn get_all() -> Vec<GamedataVerificationType> {
    Self::ALL.to_vec()
  }

  pub fn run(
    self,
    project: &GamedataProject,
    options: &GamedataProjectVerifyOptions,
  ) -> GamedataVerificationCheckReport {
    match self {
      Self::Collisions => Self::check_report(self, project.verify_collisions(options)),
      Self::Coverage => Self::check_report(self, project.verify_coverage(options)),
      Self::Animations => Self::check_report(self, project.verify_animations(options)),
      Self::Levels => Self::check_report(self, project.verify_levels(options)),
      Self::Ltx => Self::check_report(self, project.verify_ltx(options)),
      Self::Meshes => Self::check_report(self, project.verify_meshes(options)),
      Self::Particles => Self::check_report(self, project.verify_particles(options)),
      Self::ParticlesUsage => Self::check_report(self, project.verify_particles_usage(options)),
      Self::Scripts => Self::check_report(self, project.verify_scripts(options)),
      Self::Shaders => Self::check_report(self, project.verify_shaders(options)),
      Self::Sounds => Self::check_report(self, project.verify_sounds(options)),
      Self::Spawns => Self::check_report(self, project.verify_spawns(options)),
      Self::Textures => Self::check_report(self, project.verify_textures(options)),
      Self::Weapons => Self::check_report(self, project.verify_weapons(options)),
      Self::Weathers => Self::check_report(self, project.verify_weathers(options)),
    }
  }

  fn check_report<T>(verification_type: Self, result: xrf_error::XrfResult<T>) -> GamedataVerificationCheckReport
  where
    T: GamedataCheckResult,
  {
    GamedataVerificationCheckReport::from_check_result(verification_type, result)
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
