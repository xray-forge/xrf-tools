use std::collections::HashSet;

use xrf_output::OutputOptions;

use crate::project::gamedata_verification_type::GamedataVerificationType;

#[derive(Clone, Default)]
pub struct GamedataProjectVerifyOptions {
  pub output: OutputOptions,
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub is_strict: bool,
  pub checks: Vec<GamedataVerificationType>,
}

impl GamedataProjectVerifyOptions {
  /// The same options, saying what this worker says through `output`.
  pub fn with_output(&self, output: OutputOptions) -> Self {
    Self { output, ..self.clone() }
  }

  pub fn selected_checks(&self) -> Vec<GamedataVerificationType> {
    let mut seen: HashSet<GamedataVerificationType> = HashSet::with_capacity(self.checks.len());

    self
      .checks
      .iter()
      .copied()
      // Coverage and collisions always run first, so a caller naming either programmatically must not run it again.
      .filter(|check| {
        !matches!(
          check,
          GamedataVerificationType::Coverage | GamedataVerificationType::Collisions
        )
      })
      .filter(|check| seen.insert(*check))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataProjectVerifyOptions;
  use crate::GamedataVerificationType;

  #[test]
  fn selected_checks_preserves_first_requested_order_and_removes_duplicates() {
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![
        GamedataVerificationType::Textures,
        GamedataVerificationType::Scripts,
        GamedataVerificationType::Textures,
      ],
      ..Default::default()
    };

    assert_eq!(
      options.selected_checks(),
      vec![GamedataVerificationType::Textures, GamedataVerificationType::Scripts,]
    );
  }

  /// The two always-run checks already ran before the selection, so naming them must not run them a second time.
  #[test]
  fn selected_checks_leaves_out_the_always_run_checks() {
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![
        GamedataVerificationType::Coverage,
        GamedataVerificationType::Ltx,
        GamedataVerificationType::Collisions,
      ],
      ..Default::default()
    };

    assert_eq!(options.selected_checks(), vec![GamedataVerificationType::Ltx]);
  }
}
