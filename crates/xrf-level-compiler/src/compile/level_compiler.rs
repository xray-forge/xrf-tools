use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};
use xrf_level::{BuildCformFile, BuildPrjFile, LevelLightsFile};
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::format_path;

use crate::ai::ai_step::AiStep;
use crate::compile::level_compile_report::LevelCompileReport;
use crate::compile::level_compile_request::LevelCompileRequest;
use crate::compile::level_compile_step::LevelCompileStep;
use crate::compile::level_compile_step_report::LevelCompileStepReport;
use crate::compile::level_compile_step_run::LevelCompileStepRun;
use crate::detail::detail_step::DetailStep;
use crate::geometry::level::geometry_step::GeometryStep;
use crate::geometry::level::geometry_step_run::GeometryStepRun;
use crate::input::compiler_textures::CompilerTextures;
use crate::light::bake::light_bake::LightBake;
use crate::ray::ray_tracing::RayTracing;

/// Compiles a level from the files `level build` wrote, step by step ([`LevelCompileStep`]): every step's inputs are
/// checked before any runs, so a missing file stops the compile at once rather than an hour in; then each runs in order
/// in this process, reported to a job and stopped when it is cancelled. The geometry step compiles the level's geometry
/// (`GeometryLevel`), baking its light in the same pass where the light step runs, which reports the bake; the details
/// step lights the details ([`crate::DetailLighter`]) and the AI step compiles the AI map ([`crate::AiMapCompiler`]).
pub struct LevelCompiler;

impl LevelCompiler {
  /// What a job reports a compile under: its steps, each named as its detail.
  pub(crate) const PROGRESS_STEPS: &'static str = "steps";

  /// Runs the request's steps on its level.
  ///
  /// # Errors
  ///
  /// Returns an error before anything runs when the light step is asked for without the geometry step, the level's
  /// directory has no name, a step's input or the compiler shader library is missing, a texture `build.prj` names has
  /// no `.thm` under the textures roots where a step reads them, or the details or AI step would overwrite
  /// `level.details` or `level.ai` unforced; and while running when a step fails or the job is cancelled.
  pub fn compile(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<LevelCompileReport> {
    let steps: Vec<LevelCompileStep> = Self::get_steps(&request.steps);

    if let Some(step) = steps.iter().find(|step| {
      step
        .get_companion()
        .is_some_and(|companion| !steps.contains(&companion))
    }) {
      return Err(XrfError::new_invalid_error(format!(
        "The {} step runs only with the {} step, in the same compile",
        step.get_name(),
        step.get_companion().map_or("", LevelCompileStep::get_name)
      )));
    }

    // The geometry step bakes the light in its own pass where the light step runs.
    let light: Option<RayTracing<'_>> = steps.contains(&LevelCompileStep::Light).then(|| request.get_tracing());

    Self::check_level_name(&request.level)?;
    Self::check_inputs(request, &steps)?;

    let scope: JobScope = job.enter(Self::PROGRESS_STEPS, Some(steps.len() as u64));
    let mut reports: Vec<LevelCompileStepReport> = Vec::with_capacity(steps.len());
    // The light the geometry step baked, which the light step reports.
    let mut baked: Option<LightBake> = None;

    for step in &steps {
      job.check_cancelled()?;
      job.set_detail(Some(String::from(step.get_name())));

      let started: Instant = Instant::now();
      let (duration, run): (Duration, LevelCompileStepRun) = match step {
        LevelCompileStep::Geometry => {
          let GeometryStepRun { run, bake } = GeometryStep::run(request, job, light)?;
          // The bake ran within the geometry's compile; its time is the light step's.
          let light: Duration = bake.as_ref().map_or(Duration::ZERO, |bake| bake.duration);

          baked = bake;

          (started.elapsed().saturating_sub(light), run)
        }
        LevelCompileStep::Light => {
          let bake: LightBake = baked
            .take()
            .ok_or_else(|| XrfError::new_unexpected_error("The light step ran with no light baked"))?;

          (
            bake.duration,
            LevelCompileStepRun::LightBaker {
              census: bake.census,
              gpu: bake.gpu,
            },
          )
        }
        LevelCompileStep::Details => {
          let run: LevelCompileStepRun = DetailStep::run(request, job)?;

          (started.elapsed(), run)
        }
        LevelCompileStep::Ai => {
          let run: LevelCompileStepRun = AiStep::run(request, job)?;

          (started.elapsed(), run)
        }
      };

      reports.push(LevelCompileStepReport {
        step: *step,
        duration: u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        run,
      });
      scope.advance();
    }

    Ok(LevelCompileReport {
      level: format_path(&request.level).to_string(),
      steps: reports,
    })
  }

  /// `steps`, each once in the order a compile runs them; every step where none is given.
  pub(crate) fn get_steps(steps: &[LevelCompileStep]) -> Vec<LevelCompileStep> {
    LevelCompileStep::ALL
      .into_iter()
      .filter(|step| steps.is_empty() || steps.contains(step))
      .collect()
  }

  /// Whether `level` ends in a level directory's name, which the level is known by.
  fn check_level_name(level: &Path) -> XrfResult {
    level
      .file_name()
      .and_then(|name| name.to_str())
      .filter(|name| !name.is_empty())
      .map(|_| ())
      .ok_or_else(|| XrfError::new_invalid_error(format!("{} names no level directory", format_path(level))))
  }

  /// Whether every file `steps` read is there, those an earlier step of the same compile writes left aside; every
  /// texture `build.prj` names has its `.thm` where a step reads them, as each of the compilers loads and asserts on
  /// (`xrAI/compiler_load.cpp` for one); and no step of ours writes over its output unforced.
  fn check_inputs(request: &LevelCompileRequest, steps: &[LevelCompileStep]) -> XrfResult {
    let written_by_geometry: [&str; 2] = [BuildCformFile::FILE_NAME, LevelLightsFile::FILE_NAME];
    let is_geometry_run: bool = steps.contains(&LevelCompileStep::Geometry);

    for step in steps {
      for input in step.get_inputs(request.ai_covers) {
        if is_geometry_run && *step != LevelCompileStep::Geometry && written_by_geometry.contains(input) {
          continue;
        }

        if !request.level.join(input).is_file() {
          return Err(XrfError::new_not_found_error(format!(
            "The {} step needs {}, which {} does not hold",
            step.get_name(),
            input,
            format_path(&request.level)
          )));
        }
      }
    }

    if let Some(step) = steps.iter().find(|step| step.is_reading_library(request.ai_covers)) {
      Self::check_library(request, *step)?;
    }

    if steps.contains(&LevelCompileStep::Details) {
      Self::check_output(request, DetailStep::OUTPUT)?;
    }

    if steps.contains(&LevelCompileStep::Ai) {
      Self::check_output(request, AiStep::OUTPUT)?;
    }

    if !steps.iter().any(|step| step.is_reading_textures(request.ai_covers)) {
      return Ok(());
    }

    let project: BuildPrjFile =
      BuildPrjFile::read_from_path::<XRayByteOrder, _>(&request.level.join(BuildPrjFile::FILE_NAME))?;
    let roots: Vec<PathBuf> = request.get_textures_roots();
    let missing: Vec<String> = project
      .textures
      .iter()
      .map(|texture| CompilerTextures::get_name(&texture.name.value))
      .filter(|name| name != CompilerTextures::UNLOADED)
      .filter(|name| CompilerTextures::find(&roots, name, "thm").is_none())
      .collect();

    match missing.as_slice() {
      [] => Ok(()),
      [first, ..] => Err(XrfError::new_not_found_error(format!(
        "{} of the textures build.prj names have no .thm under {}, which the compilers need, such as {first}",
        missing.len(),
        Self::describe_roots(&roots)
      ))),
    }
  }

  /// Whether the game data holds the compiler shader library ([`LevelCompileStep::is_reading_library`]), `step` naming
  /// the first step reading it in a message.
  fn check_library(request: &LevelCompileRequest, step: LevelCompileStep) -> XrfResult {
    let library: PathBuf = request.gamedata.join(ShaderCompilerFile::FILE_NAME);

    if library.is_file() {
      return Ok(());
    }

    Err(XrfError::new_not_found_error(format!(
      "The {} step needs {}, which is not there",
      step.get_name(),
      format_path(&library)
    )))
  }

  /// Whether a step of ours can write `name` in the level's directory: it holds no such file, or the request is forced.
  fn check_output(request: &LevelCompileRequest, name: &str) -> XrfResult {
    let output: PathBuf = request.level.join(name);

    if output.exists() && !request.is_forced {
      return Err(XrfError::new_invalid_error(format!(
        "{} exists; force the compile to overwrite it",
        format_path(&output)
      )));
    }

    Ok(())
  }

  /// The textures roots as a message names them.
  fn describe_roots(roots: &[PathBuf]) -> String {
    roots
      .iter()
      .map(|root| format_path(root).to_string())
      .collect::<Vec<_>>()
      .join(", ")
  }
}
