use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};
use xrf_level::BuildPrjFile;
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::format_path;

use crate::compiler_textures::CompilerTextures;
use crate::detail::detail_step::DetailStep;
use crate::level_compile_report::{LevelCompileReport, LevelCompileStepReport};
use crate::level_compile_request::LevelCompileRequest;
use crate::level_compile_step::LevelCompileStep;
use crate::level_compile_step_run::LevelCompileStepRun;
use crate::sdk::level_sdk_process::LevelSdkProcess;
use crate::sdk::level_sdk_program::LevelSdkProgram;
use crate::sdk::level_sdk_workspace::LevelSdkWorkspace;

/// Compiles a level from the files `level build` wrote, step by step ([`LevelCompileStep`]): each step's inputs and
/// program are checked before any runs, so a missing file stops the compile at once rather than an hour in, then each
/// runs in order, reported to a job: the details by [`crate::DetailLighter`] here, cancelled with the job; the others
/// by the SDK's compilers in a workspace of their own, killed when the job is cancelled.
pub struct LevelCompiler;

impl LevelCompiler {
  /// What a job reports a compile under: its steps, each named as its detail until the compiler's log says more.
  pub const PROGRESS_STEPS: &'static str = "steps";

  /// Runs the request's steps on its level.
  ///
  /// # Errors
  ///
  /// Returns an error before anything runs when the level's directory has no name, a step's input is missing, a
  /// texture `build.prj` names has no `.thm` under the textures roots, the details step would overwrite `level.details`
  /// unforced, or a step the SDK runs has no SDK, no such program in it, or other than one textures root; and while
  /// running when a step fails or the job is cancelled.
  pub fn compile(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<LevelCompileReport> {
    let steps: Vec<LevelCompileStep> = Self::get_steps(&request.steps);
    let (levels, name) = Self::get_level_name(&request.level)?;
    let programs: Vec<Option<LevelSdkProgram>> = steps
      .iter()
      .map(|step| Self::get_program(request, *step))
      .collect::<XrfResult<_>>()?;

    if programs.iter().any(Option::is_some) {
      Self::get_sdk_textures(request)?;
    }

    Self::check_inputs(request, &steps)?;

    let work: PathBuf = request
      .work
      .clone()
      .unwrap_or_else(|| std::env::temp_dir().join(format!("xrf-level-compile-{name}")));
    // Made by the first step the SDK runs, and only then.
    let mut workspace: Option<LevelSdkWorkspace> = None;
    let scope: JobScope = job.enter(Self::PROGRESS_STEPS, Some(steps.len() as u64));
    let mut reports: Vec<LevelCompileStepReport> = Vec::with_capacity(steps.len());

    for (step, program) in steps.iter().zip(&programs) {
      job.check_cancelled()?;
      job.set_detail(Some(String::from(step.get_name())));

      let (duration, run): (Duration, LevelCompileStepRun) = match program {
        Some(program) => {
          let made: LevelSdkWorkspace = match workspace.take() {
            Some(made) => made,
            None => LevelSdkWorkspace::create(&work, &levels, &request.gamedata, Self::get_sdk_textures(request)?)?,
          };
          let workspace: &LevelSdkWorkspace = workspace.insert(made);
          let since: SystemTime = SystemTime::now();
          let duration: Duration = LevelSdkProcess::run(program, &name, workspace, job)?;

          (
            duration,
            LevelCompileStepRun::Program {
              program: format_path(&program.path).to_string(),
              log: workspace
                .find_log(program.log_prefix, since)
                .map(|log| format_path(&log).to_string()),
            },
          )
        }
        // A step of ours, the details being the one so far.
        None => {
          let started: Instant = Instant::now();
          let run: LevelCompileStepRun = DetailStep::run(request, job)?;

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
      work: workspace.map(|workspace| format_path(&workspace.directory).to_string()),
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

  /// The SDK's program running `step`, none for a step of ours.
  fn get_program(request: &LevelCompileRequest, step: LevelCompileStep) -> XrfResult<Option<LevelSdkProgram>> {
    match &request.sdk {
      _ if !step.is_sdk() => Ok(None),
      Some(sdk) => LevelSdkProgram::of(sdk, step),
      None => Err(XrfError::new_invalid_error(format!(
        "The {} step runs the SDK's compiler; name the SDK's editors directory",
        step.get_name()
      ))),
    }
  }

  /// The one textures root the SDK's compilers read, `$game_textures$`.
  fn get_sdk_textures(request: &LevelCompileRequest) -> XrfResult<&Path> {
    match request.textures.as_slice() {
      [root] => Ok(root),
      roots => Err(XrfError::new_invalid_error(format!(
        "The SDK's compilers read one textures root, not {}",
        roots.len()
      ))),
    }
  }

  /// Whether every file `steps` read is there, those an earlier step of the same compile writes left aside; every
  /// texture `build.prj` names has its `.thm`, which each of the compilers loads and asserts on
  /// (`xrAI/compiler_load.cpp:140` for one); and the details step writes no `level.details` over one unforced.
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

    if steps.contains(&LevelCompileStep::Details) {
      Self::check_details(request)?;
    }

    let project: BuildPrjFile =
      BuildPrjFile::read_from_path::<XRayByteOrder, _>(&request.level.join(BuildPrjFile::FILE_NAME))?;
    let missing: Vec<String> = project
      .textures
      .iter()
      .map(|texture| CompilerTextures::get_name(&texture.name.value))
      .filter(|name| name != CompilerTextures::UNLOADED)
      .filter(|name| CompilerTextures::find(&request.textures, name, "thm").is_none())
      .collect();

    match missing.as_slice() {
      [] => Ok(()),
      [first, ..] => Err(XrfError::new_not_found_error(format!(
        "{} of the textures build.prj names have no .thm under {}, which the compilers need, such as {first}",
        missing.len(),
        Self::describe_roots(&request.textures)
      ))),
    }
  }

  /// Whether the details step can write: the game data holds the compiler shader library it reads, and the level's
  /// directory no `level.details` but where the request is forced.
  fn check_details(request: &LevelCompileRequest) -> XrfResult {
    let library: PathBuf = request.gamedata.join(ShaderCompilerFile::FILE_NAME);
    let output: PathBuf = request.level.join(DetailStep::OUTPUT);

    if !library.is_file() {
      return Err(XrfError::new_not_found_error(format!(
        "The details step needs {}, which is not there",
        format_path(&library)
      )));
    }

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
    match roots {
      [] => String::from("no textures root"),
      roots => roots
        .iter()
        .map(|root| format_path(root).to_string())
        .collect::<Vec<_>>()
        .join(", "),
    }
  }
}
