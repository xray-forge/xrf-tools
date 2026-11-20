use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};
use xrf_level::{BuildCformFile, BuildPrjFile, LevelLightsFile};
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::{OutputFiles, format_path};

use crate::ai::ai_step::AiStep;
use crate::compile::level_compile_report::LevelCompileReport;
use crate::compile::level_compile_request::LevelCompileRequest;
use crate::compile::level_compile_step::LevelCompileStep;
use crate::compile::level_compile_step_report::LevelCompileStepReport;
use crate::compile::level_compile_step_run::LevelCompileStepRun;
use crate::detail::detail_step::DetailStep;
use crate::geometry::level::geometry_step::GeometryStep;
use crate::geometry::level::geometry_step_run::GeometryStepRun;
use crate::input::compiler_texture::CompilerTexture;
use crate::input::compiler_textures::CompilerTextures;
use crate::light::bake::light_bake::LightBake;
use crate::ray::ray_tracing::RayTracing;

/// Compiles a level from the files `level build` wrote, step by step ([`LevelCompileStep`]), in this process.
///
/// Every step's inputs are checked before any runs, so a missing file stops the compile at once rather than an hour in.
/// The geometry step bakes the light in its own pass where the light step runs; the light step only reports that bake.
pub struct LevelCompiler;

impl LevelCompiler {
  /// What a job reports a compile under: its steps, each named as its detail.
  pub(crate) const PROGRESS_STEPS: &'static str = "steps";

  /// Runs the request's steps on its level.
  ///
  /// # Errors
  ///
  /// Returns an error before any step runs when the light step is asked for without the geometry step, the level's
  /// directory has no name, a step's input or `shaders_xrlc.xr` is missing, a texture lacks its `.thm` where a step
  /// reads textures, or an unforced request would overwrite a file; later, when a step fails or the job is cancelled.
  pub fn compile(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<LevelCompileReport> {
    let steps: Vec<LevelCompileStep> = Self::get_steps(&request.steps);

    Self::check_files(request, &steps, &[], &[])?;

    let project: Option<BuildPrjFile> = steps
      .iter()
      .any(|step| step.is_reading_textures(request.ai_covers))
      .then(|| BuildPrjFile::read_from_path::<XRayByteOrder, _>(&request.level.join(BuildPrjFile::FILE_NAME)))
      .transpose()?;

    Self::check_textures_and_outputs(request, &steps, project.as_ref())?;

    let light: Option<RayTracing<'_>> = steps.contains(&LevelCompileStep::Light).then(|| request.get_tracing());

    let scope: JobScope = job.enter(Self::PROGRESS_STEPS, Some(steps.len() as u64));
    let mut reports: Vec<LevelCompileStepReport> = Vec::with_capacity(steps.len());
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

  /// The checks of [`Self::compile`] for a job that may write `built` into the level's directory first, before it knows
  /// which: each of `built` taken as there, and the textures unchecked since `build.prj` is not yet made. A quick
  /// refusal before the build's work; [`Self::check_build`] follows once the build is made.
  ///
  /// # Errors
  ///
  /// Returns an error where the compile would refuse, the textures aside.
  pub fn check_before_writing(request: &LevelCompileRequest, built: &[&str]) -> XrfResult {
    let steps: Vec<LevelCompileStep> = Self::get_steps(&request.steps);

    Self::check_files(request, &steps, built, built)?;
    Self::get_outputs(request, &steps, &[]).assert_free(request.is_forced)
  }

  /// The checks of [`Self::compile`] for a build made but not yet written into the level's directory: `project` its
  /// `build.prj`, `written` the files it writes, and the rest of `owned`, the files it may write, gone once it has, as
  /// a build removes or refuses its own files it does not write.
  ///
  /// # Errors
  ///
  /// Returns an error where the compile would refuse once the build is written.
  pub fn check_build(
    request: &LevelCompileRequest,
    project: &BuildPrjFile,
    written: &[&str],
    owned: &[&str],
  ) -> XrfResult {
    let steps: Vec<LevelCompileStep> = Self::get_steps(&request.steps);

    Self::check_files(request, &steps, written, owned)?;
    Self::check_textures_and_outputs(request, &steps, Some(project))
  }

  /// `steps`, each once in the order a compile runs them; every step where none is given.
  pub(crate) fn get_steps(steps: &[LevelCompileStep]) -> Vec<LevelCompileStep> {
    LevelCompileStep::ALL
      .into_iter()
      .filter(|step| steps.is_empty() || steps.contains(step))
      .collect()
  }

  /// Whether `level` ends in a directory name, which the level is known by.
  fn check_level_name(level: &Path) -> XrfResult {
    level
      .file_name()
      .and_then(|name| name.to_str())
      .filter(|name| !name.is_empty())
      .map(|_| ())
      .ok_or_else(|| XrfError::new_invalid_error(format!("{} names no level directory", format_path(level))))
  }

  /// Whether `steps` can run together, the level's directory is named, every input is there (those `written` first or
  /// by an earlier step taken as there, the rest of `owned` as missing), and the game data holds `shaders_xrlc.xr`
  /// where a step reads it.
  fn check_files(
    request: &LevelCompileRequest,
    steps: &[LevelCompileStep],
    written: &[&str],
    owned: &[&str],
  ) -> XrfResult {
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

    Self::check_level_name(&request.level)?;

    let written_by_geometry: [&str; 2] = [BuildCformFile::FILE_NAME, LevelLightsFile::FILE_NAME];
    let is_geometry_run: bool = steps.contains(&LevelCompileStep::Geometry);

    for step in steps {
      for input in step.get_inputs(request.ai_covers) {
        if (is_geometry_run && *step != LevelCompileStep::Geometry && written_by_geometry.contains(input))
          || written.contains(input)
        {
          continue;
        }

        if owned.contains(input) {
          return Err(XrfError::new_not_found_error(format!(
            "The {} step needs {}, which the build does not write",
            step.get_name(),
            input
          )));
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

    match steps.iter().find(|step| step.is_reading_library(request.ai_covers)) {
      Some(step) => Self::check_library(request, *step),
      None => Ok(()),
    }
  }

  /// Whether every texture of `project`, the level's `build.prj`, has the `.thm` the compilers assert on, where a step
  /// reads textures; and, unforced, no file a step writes is already there.
  pub(crate) fn check_textures_and_outputs(
    request: &LevelCompileRequest,
    steps: &[LevelCompileStep],
    project: Option<&BuildPrjFile>,
  ) -> XrfResult {
    let mut implicit: Vec<String> = Vec::new();

    if let Some(project) = project.filter(|_| steps.iter().any(|step| step.is_reading_textures(request.ai_covers))) {
      let roots: Vec<PathBuf> = request.get_textures_roots();

      Self::check_textures(project, &roots)?;

      // The geometry step writes every implicitly lit texture's maps: baked with the light step, neutral without it.
      if steps.contains(&LevelCompileStep::Geometry) {
        implicit = CompilerTexture::resolve_all(project, &roots)?
          .into_iter()
          .filter(|texture| texture.is_implicitly_lit)
          .map(|texture| texture.name)
          .collect();
      }
    }

    Self::get_outputs(request, steps, &implicit).assert_free(request.is_forced)
  }

  /// Whether every texture of `project` names a file below the roots and has its `.thm` under one of them.
  fn check_textures(project: &BuildPrjFile, roots: &[PathBuf]) -> XrfResult {
    let mut missing: Vec<String> = Vec::new();

    for texture in &project.textures {
      let name: String = CompilerTextures::get_name(&texture.name.value);

      if name != CompilerTextures::UNLOADED && CompilerTextures::find(roots, &name, "thm")?.is_none() {
        missing.push(name);
      }
    }

    match missing.as_slice() {
      [] => Ok(()),
      [first, ..] => Err(XrfError::new_not_found_error(format!(
        "{} of the textures build.prj names have no .thm under {}, which the compilers need, such as {first}",
        missing.len(),
        Self::describe_roots(roots)
      ))),
    }
  }

  /// The files `steps` write in the level's directory, the `implicit` textures' maps among them.
  pub(crate) fn get_outputs(
    request: &LevelCompileRequest,
    steps: &[LevelCompileStep],
    implicit: &[String],
  ) -> OutputFiles {
    let mut outputs: OutputFiles = OutputFiles::new(&request.level);

    if steps.contains(&LevelCompileStep::Geometry) {
      outputs = outputs
        .with_files(GeometryStep::OUTPUTS)
        .with_family(GeometryStep::LIGHTMAP_FAMILY);
    }

    for name in implicit
      .iter()
      .flat_map(|name| GeometryStep::get_implicit_textures(name))
    {
      outputs = outputs.with_file(&format!("{name}.dds"));
    }

    if steps.contains(&LevelCompileStep::Details) {
      outputs = outputs.with_file(DetailStep::OUTPUT);
    }

    if steps.contains(&LevelCompileStep::Ai) {
      outputs = outputs.with_file(AiStep::OUTPUT);
    }

    outputs
  }

  /// Whether the game data holds `shaders_xrlc.xr`, `step` being the one the error names.
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

  /// The textures roots as a message names them.
  fn describe_roots(roots: &[PathBuf]) -> String {
    roots
      .iter()
      .map(|root| format_path(root).to_string())
      .collect::<Vec<_>>()
      .join(", ")
  }
}
