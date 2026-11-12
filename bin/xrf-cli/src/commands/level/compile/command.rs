use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_level_compiler::{
  LevelCompileReport, LevelCompileRequest, LevelCompileStep, LevelCompileStepRun, LevelCompiler,
};
use xrf_output::OutputOptions;

use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Compiles a level directory `level build` wrote, step by step: the details by our lighter, the geometry and the AI
/// map by the SDK's compilers until ours replace them.
#[derive(Default)]
pub(crate) struct CompileCommand;

impl GenericCommand for CompileCommand {
  fn operation(&self) -> &'static str {
    "compile"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Compile a level directory level build wrote into what the game loads, step by step: geometry (the level, its \
         geometry, collision and lightmaps; the SDK's xrLC), details (level.details lit from build.details, \
         build.cform and build.lights, as xrDO_Light lights it, on every core) and ai (level.ai; the SDK's xrAI -f). \
         Every step's inputs and every texture build.prj names are checked before any step runs. The SDK's compilers \
         run in a working directory under an fsgame.ltx written for them, never in a game installation",
      )
      .arg(
        Arg::new("path")
          .help("Path to the level's directory, <levels>/<level>, holding build.prj and the rest level build wrote")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("steps")
          .help("Steps to run, in their order whatever the order given; every step by default")
          .long("steps")
          .value_delimiter(',')
          .value_parser(LevelCompileStep::ALL.map(LevelCompileStep::get_name).to_vec()),
      )
      .arg(
        Arg::new("sdk")
          .help("The SDK's editors directory, whose bin holds xrLC and xrAI; needed by the geometry and ai steps")
          .long("sdk")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("gamedata")
          .help(
            "Game data the compilers read gamemtl.xr and shaders_xrlc.xr from; the gamedata holding the level's \
             levels directory by default",
          )
          .long("gamedata")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("textures")
          .help(
            "Textures root with a .thm for every texture build.prj names, and the images of those with alpha; \
             repeatable, each file from the first root holding it, though the SDK's steps take one; the game data's \
             textures by default",
          )
          .long("textures")
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("work")
          .help(
            "Directory the SDK's compilers run in, with their fsgame.ltx and logs; one in the system's temporary \
             directory",
          )
          .long("work")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help("Overwrite level.details where the level's directory already holds it")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let level: PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("the path is required")
      .clone();
    let gamedata: PathBuf = matches
      .get_one::<PathBuf>("gamedata")
      .cloned()
      .unwrap_or_else(|| Self::get_gamedata(&level));
    let textures: Vec<PathBuf> = matches
      .get_many::<PathBuf>("textures")
      .map(|roots| roots.cloned().collect())
      .unwrap_or_else(|| vec![gamedata.join("textures")]);
    let request: LevelCompileRequest = LevelCompileRequest {
      steps: matches
        .get_many::<String>("steps")
        .map(|steps| steps.filter_map(|step| LevelCompileStep::from_name(step)).collect())
        .unwrap_or_default(),
      sdk: matches.get_one::<PathBuf>("sdk").cloned(),
      textures,
      gamedata,
      work: matches.get_one::<PathBuf>("work").cloned(),
      is_forced: matches.get_flag("force"),
      level,
    };
    let output: OutputOptions = context.get_output().clone();
    let report: LevelCompileReport = Self::compile(&output, &request)?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl CompileCommand {
  /// The `gamedata` directory holding `levels\<level>`, named by the level path's parents where it has them.
  fn get_gamedata(level: &Path) -> PathBuf {
    match level.parent().and_then(Path::parent) {
      Some(gamedata) if !gamedata.as_os_str().is_empty() => gamedata.to_path_buf(),
      _ => level.join("..").join(".."),
    }
  }

  fn compile(output: &OutputOptions, request: &LevelCompileRequest) -> XrfResult<LevelCompileReport> {
    let report: LevelCompileReport = LevelCompiler::compile(request, &CliJob::for_output(output))?;

    for step in &report.steps {
      let run: String = match &step.run {
        LevelCompileStepRun::Program { program, log } => format!(
          "{program}{}",
          log.as_ref().map(|log| format!(", log {log}")).unwrap_or_default()
        ),
        LevelCompileStepRun::DetailLighter { slots } => format!(
          "{} slots lit, {} touching no face, {} emptied, {} unplanted",
          slots.lit, slots.untouched, slots.emptied, slots.unplanted
        ),
      };

      xrf_output::info!(
        output,
        "{}: {run} in {:.1} s",
        step.step.get_name(),
        step.duration as f64 / 1000.0
      );
    }

    xrf_output::success!(
      output,
      "Compiled {} ({}){}",
      report.level,
      report
        .steps
        .iter()
        .map(|step| step.step.get_name())
        .collect::<Vec<_>>()
        .join(", "),
      report
        .work
        .as_ref()
        .map(|work| format!(", the SDK's compilers in {work}"))
        .unwrap_or_default()
    );

    Ok(report)
  }
}
