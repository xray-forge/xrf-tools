use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};
use xrf_level::BuildPrjFile;
use xrf_utils::format_path;

use crate::level_compile_report::{LevelCompileReport, LevelCompileStepReport};
use crate::level_compile_request::LevelCompileRequest;
use crate::level_compile_step::LevelCompileStep;
use crate::sdk::level_sdk_process::LevelSdkProcess;
use crate::sdk::level_sdk_program::LevelSdkProgram;
use crate::sdk::level_sdk_workspace::LevelSdkWorkspace;

/// Compiles a level from the files `level build` wrote, step by step ([`LevelCompileStep`]): each step's inputs and
/// program are checked before any runs, so a missing file stops the compile at once rather than an hour in, then each
/// runs in order in a workspace of its own, reported to a job and killed when the job is cancelled.
pub struct LevelCompiler;

impl LevelCompiler {
  /// What a job reports a compile under: its steps, each named as its detail until the compiler's log says more.
  pub const PROGRESS_STEPS: &'static str = "steps";

  /// The texture xrLC takes without loading it, the merged impostors' atlas (`xrLC/Build_Load.cpp:304`).
  const UNLOADED_TEXTURE: &'static str = "level_lods";

  /// Runs the request's steps on its level.
  ///
  /// # Errors
  ///
  /// Returns an error before anything runs when the level's directory has no name, a step's input is missing, a
  /// texture `build.prj` names has no `.thm` under the textures root, or the SDK lacks a step's program; and while
  /// running when the workspace is not written, a program fails or the job is cancelled.
  pub fn compile(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<LevelCompileReport> {
    let steps: Vec<LevelCompileStep> = Self::get_steps(&request.steps);
    let (levels, name) = Self::get_level_name(&request.level)?;
    let programs: Vec<(LevelCompileStep, LevelSdkProgram)> = steps
      .iter()
      .map(|step| Ok((*step, LevelSdkProgram::of(&request.sdk, *step)?)))
      .collect::<XrfResult<_>>()?;

    Self::check_inputs(request, &steps)?;

    let work: PathBuf = request
      .work
      .clone()
      .unwrap_or_else(|| std::env::temp_dir().join(format!("xrf-level-compile-{name}")));
    let workspace: LevelSdkWorkspace = LevelSdkWorkspace::create(&work, &levels, &request.gamedata, &request.textures)?;
    let scope: JobScope = job.enter(Self::PROGRESS_STEPS, Some(programs.len() as u64));
    let mut reports: Vec<LevelCompileStepReport> = Vec::with_capacity(programs.len());

    for (step, program) in &programs {
      job.check_cancelled()?;
      job.set_detail(Some(String::from(step.get_name())));

      let since: SystemTime = SystemTime::now();
      let duration: Duration = LevelSdkProcess::run(program, &name, &workspace, job)?;

      reports.push(LevelCompileStepReport {
        step: *step,
        program: format_path(&program.path).to_string(),
        duration: u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        log: workspace
          .find_log(program.log_prefix, since)
          .map(|log| format_path(&log).to_string()),
      });
      scope.advance();
    }

    Ok(LevelCompileReport {
      level: format_path(&request.level).to_string(),
      work: format_path(&work).to_string(),
      steps: reports,
    })
  }

  /// `steps`, each once in the order a compile runs them; every step where none is given.
  pub fn get_steps(steps: &[LevelCompileStep]) -> Vec<LevelCompileStep> {
    LevelCompileStep::ALL
      .into_iter()
      .filter(|step| steps.is_empty() || steps.contains(step))
      .collect()
  }

  /// The levels directory holding `level` and the level's name, its directory's.
  fn get_level_name(level: &Path) -> XrfResult<(PathBuf, String)> {
    let name: String = level
      .file_name()
      .and_then(|name| name.to_str())
      .filter(|name| !name.is_empty())
      .ok_or_else(|| XrfError::new_invalid_error(format!("{} names no level directory", format_path(level))))?
      .to_owned();
    let levels: PathBuf = level
      .parent()
      .filter(|parent| !parent.as_os_str().is_empty())
      .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    Ok((levels, name))
  }

  /// Whether every file `steps` read is there, those an earlier step of the same compile writes left aside, and every
  /// texture `build.prj` names has its `.thm`, which each of the three compilers loads and asserts on
  /// (`xrAI/compiler_load.cpp:140` for one).
  fn check_inputs(request: &LevelCompileRequest, steps: &[LevelCompileStep]) -> XrfResult {
    let written_by_geometry: [&str; 2] = ["build.cform", "build.lights"];
    let is_geometry_run: bool = steps.contains(&LevelCompileStep::Geometry);

    for step in steps {
      for input in step.get_inputs() {
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

    let project: BuildPrjFile = BuildPrjFile::read_from_path::<XRayByteOrder, _>(&request.level.join("build.prj"))?;
    let missing: Vec<&str> = project
      .textures
      .iter()
      .map(|texture| texture.name.value.as_str())
      .filter(|name| !name.eq_ignore_ascii_case(Self::UNLOADED_TEXTURE))
      .filter(|name| !Self::get_texture_path(&request.textures, name, "thm").is_file())
      .collect();

    match missing.as_slice() {
      [] => Ok(()),
      [first, ..] => Err(XrfError::new_not_found_error(format!(
        "{} of the textures build.prj names have no .thm under {}, which xrLC needs, such as {first}",
        missing.len(),
        format_path(&request.textures)
      ))),
    }
  }

  /// A texture's file under the textures root, `<root>\<name>.<extension>`.
  fn get_texture_path(textures: &Path, name: &str, extension: &str) -> PathBuf {
    name
      .split(['\\', '/'])
      .fold(textures.to_path_buf(), |path, part| path.join(part))
      .with_extension(extension)
  }
}
