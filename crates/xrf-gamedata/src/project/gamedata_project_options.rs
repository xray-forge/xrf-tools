use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_ltx::LtxDialect;
use xrf_output::OutputOptions;

use crate::project::gamedata_verification_type::GamedataVerificationType;

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
  /// Which rules resolve this project's configs.
  ///
  /// Standard LTX unless a caller says otherwise. Named here rather than inferred, because a patched Anomaly tree and
  /// a vanilla one look alike from the outside and resolve differently.
  pub dialect: Arc<dyn LtxDialect>,
}

impl Default for GamedataProjectReadOptions {
  fn default() -> Self {
    Self {
      dialect: Arc::new(xrf_ltx::LtxStandardDialect),
      ignored: Vec::new(),
      is_strict: false,
      is_tracing_reads: false,
      output: OutputOptions::default(),
      root: PathBuf::new(),
    }
  }
}

#[derive(Clone, Default)]
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
  /// The same options, saying what this worker says through `output`.
  ///
  /// How parallel work keeps its output in the order the work was listed: a worker is handed a slot of an
  /// [`xrf_output::OutputSequence`] rather than the shared sink, so what it says is released in its listed position
  /// instead of when it happened to finish.
  pub fn with_output(&self, output: OutputOptions) -> Self {
    Self { output, ..self.clone() }
  }

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
