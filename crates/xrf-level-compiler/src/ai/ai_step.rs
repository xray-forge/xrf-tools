use std::path::{Path, PathBuf};

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_level::{BuildAimapFile, LevelAiFile, LevelAiGraphVerifier, LevelAiGraphVerifyResult};
use xrf_report::Status;
use xrf_utils::{StagedFile, format_path};

use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_compilation::AiMapCompilation;
use crate::ai::ai_map_compiler::AiMapCompiler;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::ai_map_request::AiMapRequest;
use crate::ai::cover::ai_ray_model::AiRayModel;
use crate::level_compile_request::LevelCompileRequest;
use crate::level_compile_step_run::LevelCompileStepRun;
use crate::ray::ray_model::RayModel;

/// A compile's AI step, xrAI `-f`'s part: the level's `level.ai` compiled from `build.aimap` by [`AiMapCompiler`], its
/// covers traced against the ray model (`build.cform`) or left out as the request says, its identity kept from the
/// `level.ai` it replaces while the graph stands; then checked as a graph (`LevelAiGraphVerifier`) before it is
/// written, a node linked to itself refusing the map.
pub(crate) struct AiStep;

impl AiStep {
  /// The step's one output, which it refuses to overwrite unless the request is forced.
  pub(crate) const OUTPUT: &'static str = LevelAiFile::FILE_NAME;

  /// Compiles the request's level's AI map and writes it, answering what the compile and the graph check found.
  ///
  /// # Errors
  ///
  /// Returns an error when an input is unreadable, the shadow materials cannot be loaded, the map does not compile or
  /// links a node to itself, the backend cannot trace, the job is cancelled, or `level.ai` is not written; a file left
  /// as it was in each case.
  pub(crate) fn run(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<LevelCompileStepRun> {
    let level: &Path = &request.level;
    let output: PathBuf = level.join(Self::OUTPUT);
    let aimap: BuildAimapFile =
      BuildAimapFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildAimapFile::FILE_NAME))?;
    let replaced: Option<AiMap> = Self::read_replaced(&output);
    let model: Option<RayModel> = match request.ai_covers {
      AiMapCovers::None => None,
      AiMapCovers::Full => Some(RayModel::read(request)?),
    };
    let compiled: AiMapCompilation = AiMapCompiler::compile(
      AiMapRequest {
        aimap: &aimap,
        covers: request.ai_covers,
        rays: model.as_ref().map(|model| AiRayModel {
          tracer: &model.tracer,
          shadows: &model.shadows,
        }),
        backend: request.backend,
        dxc: request.dxc.as_deref(),
        gpu: request.gpu,
        replaced: replaced.as_ref(),
      },
      job,
    )?;
    let checked: LevelAiGraphVerifyResult = LevelAiGraphVerifier::verify(
      &compiled.map.file.header,
      &compiled.map.nodes,
      &format_path(&output).to_string(),
    );

    if checked.report.status() == Status::Failed {
      return Err(XrfError::new_invalid_error(format!(
        "The compiled AI map fails its graph check, so {} is left as it was: {}",
        format_path(&output),
        checked
          .report
          .checks()
          .iter()
          .flat_map(|check| check.findings())
          .map(|finding| finding.message())
          .collect::<Vec<&str>>()
          .join("; ")
      )));
    }

    StagedFile::write(&output, &compiled.map.get_bytes()?).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(&output)),
        error.kind(),
      )
    })?;

    Ok(LevelCompileStepRun::AiMapCompiler {
      covers: request.ai_covers,
      gpu: compiled.gpu,
      guid: compiled.guid,
      census: checked.census.unwrap_or_default(),
      graph: checked.report,
    })
  }

  /// The `level.ai` the step replaces, whose identity it keeps while the graph stands; none where there is none, or
  /// where it cannot be read, the compiled map then taking an identity of its own.
  fn read_replaced(path: &Path) -> Option<AiMap> {
    if !path.is_file() {
      return None;
    }

    AiMap::read_from_path(path)
      .inspect_err(|error| {
        log::warn!(
          "{} is not read, so the compiled AI map takes an identity of its own: {error}",
          format_path(path)
        );
      })
      .ok()
  }
}
