use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome};
use xrf_level_compiler::{LevelCompileReport, LevelCompileRequest, LevelCompiler};
use xrf_rawdata::{LevelBuildExport, LevelBuildExporter, LevelBuildReport, LevelBuildRequest};

use crate::rawdata::level_build_result::LevelBuildResult;

/// A build and the compile after it as the app runs them in one job: the compile works on what the build wrote,
/// whatever the level compiler's form names for it.
#[derive(Debug)]
pub(crate) struct LevelBuildRun {
  pub(crate) build: LevelBuildRequest,
  /// None to build alone.
  pub(crate) compile: Option<LevelCompileRequest>,
}

impl LevelBuildRun {
  /// The run of `build` and `compile`, the compile on the build's directory, forced as the build is and tracing on
  /// D3D12 with `dxc`. What either leaves to its defaults, the objects beside the scene and the game data's textures,
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

  /// Runs the build, then the compile, on the job's thread; cancelled where the job was cancelled before either
  /// finished, the build's report kept where it was the compile that stopped.
  ///
  /// # Errors
  ///
  /// Returns an error when a compile is asked for of a build naming no game materials, which writes no build project,
  /// when the compile or the build refuses, or when either fails.
  pub(crate) fn run(&self, job: &JobHandle) -> XrfResult<LevelBuildResult> {
    if let Some(compile) = &self.compile {
      if self.build.game_materials.is_none() {
        return Err(XrfError::new_invalid_error(
          "A compile reads the build project, which the build writes only from the game materials; name them",
        ));
      }

      LevelCompiler::check_before_writing(compile, &LevelBuildExport::FILE_NAMES)?;
    }

    let report: LevelBuildReport = match LevelBuildExporter::build_to_directory::<XRayByteOrder>(&self.build, job) {
      Ok(report) => report,
      Err(_) if job.is_cancelled() => return Ok(Self::cancelled(None)),
      Err(error) => return Err(error),
    };
    let compiled: Option<LevelCompileReport> = match self
      .compile
      .as_ref()
      .map(|compile| LevelCompiler::compile(compile, job))
      .transpose()
    {
      Ok(compiled) => compiled,
      Err(_) if job.is_cancelled() => return Ok(Self::cancelled(Some(report))),
      Err(error) => {
        return Err(XrfError::new_invalid_error(format!(
          "The level was built into {}, writing {}, then its compile stopped: {error}",
          report.destination,
          report
            .files
            .iter()
            .map(|file| file.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
        )));
      }
    };

    Ok(LevelBuildResult {
      outcome: JobOutcome::Completed,
      report: Some(report),
      compile: compiled,
    })
  }

  fn cancelled(report: Option<LevelBuildReport>) -> LevelBuildResult {
    LevelBuildResult {
      outcome: JobOutcome::Cancelled,
      report,
      compile: None,
    }
  }
}
