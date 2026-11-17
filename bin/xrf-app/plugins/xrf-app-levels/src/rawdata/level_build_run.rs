use std::path::PathBuf;

use xrf_level_compiler::LevelCompileRequest;
use xrf_rawdata::LevelBuildRequest;

/// A build and the compile after it as the app runs them in one job: the compile works on what the build wrote,
/// whatever the level compiler's form names for it.
#[derive(Debug)]
pub(crate) struct LevelBuildRun {
  pub(crate) build: LevelBuildRequest,
  /// None to build alone.
  pub(crate) compile: Option<LevelCompileRequest>,
}

impl LevelBuildRun {
  /// The run of `build` and `compile`, the compile on the build's directory, overwriting as the build does and tracing
  /// on D3D12 with `dxc`. What either leaves to its defaults, the objects beside the scene and the game data's textures,
  /// their requests resolve as the CLI's do.
  pub(crate) fn new(build: LevelBuildRequest, compile: Option<LevelCompileRequest>, dxc: Option<PathBuf>) -> Self {
    let compile: Option<LevelCompileRequest> = compile.map(|compile| LevelCompileRequest {
      level: build.destination.clone(),
      is_forced: build.is_forced,
      dxc,
      ..compile
    });

    Self { build, compile }
  }
}
