use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_engine_target::XrayEngineChoice;
use xrf_ltx::LtxDialect;
use xrf_output::OutputOptions;

use crate::project::gamedata_verification_type::GamedataVerificationType;

pub struct GamedataProjectReadOptions {
  pub root: PathBuf,
  pub ignored: Vec<String>,
  pub output: OutputOptions,
  pub is_strict: bool,
  /// Whether the project accounts for every asset it physically reads.
  pub is_tracing_reads: bool,
  /// Which rules resolve this project's configs.
  pub dialect: Arc<dyn LtxDialect>,
  /// Which engine the tree is meant for, where the engines read the same configs differently; detected on `Auto`.
  pub engine: XrayEngineChoice,
}

impl GamedataProjectReadOptions {
  /// Root entries a gamedata repository keeps beside its assets, which a project opened over the repository skips
  /// unless told otherwise: version control, editor state, unpacked working copies and the repository's own documents.
  pub const DEFAULT_IGNORED: [&'static str; 8] = [
    ".git",
    ".idea",
    "particles_unpacked",
    "textures_unpacked",
    ".gitignore",
    ".gitattributes",
    "README.md",
    "LICENSE",
  ];
}

impl Default for GamedataProjectReadOptions {
  fn default() -> Self {
    Self {
      dialect: Arc::new(xrf_ltx::LtxStandardDialect),
      engine: XrayEngineChoice::default(),
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
