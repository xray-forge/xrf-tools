use std::collections::HashSet;
use std::path::PathBuf;

use xrf_output::OutputOptions;

use crate::project::gamedata_verification_type::GamedataVerificationType;

#[derive(Default)]
pub struct GamedataProjectReadOptions {
  pub root: PathBuf,
  pub ignored: Vec<String>,
  pub output: OutputOptions,
  pub is_strict: bool,
  /// Whether the project accounts for every asset it physically reads.
  ///
  /// A diagnostic rather than a mode: it puts a lock on the read path, so it is asked for by a caller measuring
  /// redundancy and off for everyone else.
  pub is_tracing_reads: bool,
}

#[derive(Default)]
pub struct GamedataProjectVerifyOptions {
  pub output: OutputOptions,
  /// Where progress goes and where cancellation comes from.
  ///
  /// Handed down into the checks that report their own work, so a run reads as `checks 3/11` over `verify 400/2000`
  /// rather than as one bar that sits still for minutes.
  pub job: xrf_job::JobHandle,
  pub is_strict: bool,
  pub checks: Vec<GamedataVerificationType>,
}

impl GamedataProjectVerifyOptions {
  pub fn selected_checks(&self) -> Vec<GamedataVerificationType> {
    let mut seen: HashSet<GamedataVerificationType> = HashSet::with_capacity(self.checks.len());

    self
      .checks
      .iter()
      .copied()
      // The collisions check always runs, so a caller naming it programmatically must not run it a second time.
      .filter(|check| *check != GamedataVerificationType::Collisions)
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
}
