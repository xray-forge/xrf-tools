use std::path::PathBuf;
use std::sync::Arc;

use xrf_engine_target::XrayEngineChoice;
use xrf_ltx::LtxDialect;
use xrf_output::OutputOptions;

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
