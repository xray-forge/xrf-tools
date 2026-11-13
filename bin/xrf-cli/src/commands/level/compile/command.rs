use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_level_compiler::{
  AiMapCovers, AiMapGuid, LevelCompileReport, LevelCompileRequest, LevelCompileStep, LevelCompileStepRun,
  LevelCompiler, RayBackend,
};
use xrf_output::OutputOptions;

use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Compiles a level directory `level build` wrote, step by step: the details by our lighter, the AI map by our
/// compiler, the geometry by the SDK's xrLC until ours replaces it.
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
         build.cform and build.lights, as xrDO_Light lights it) and ai (level.ai compiled from build.aimap as xrAI -f \
         compiles it, its covers traced against build.cform, then checked as a graph). Every step's inputs and every \
         texture build.prj names are checked before any step runs. The SDK's xrLC runs in a working directory under an \
         fsgame.ltx written for it, never in a game installation",
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
          .help("The SDK's editors directory, whose bin holds xrLC; needed by the geometry step")
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
             repeatable, each file from the first root holding it, though the SDK's xrLC takes one; the game data's \
             textures by default",
          )
          .long("textures")
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("work")
          .help(
            "Directory the SDK's xrLC runs in, with its fsgame.ltx and logs; one in the system's temporary directory",
          )
          .long("work")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help(
            "Overwrite level.details and level.ai where the level's directory already holds them; a level.ai replaced \
             keeps its identity while its graph is unchanged, so the level's spawn still loads",
          )
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("backend")
          .help(
            "What traces the details step's shadow rays and the ai step's cover rays: auto, the GPU where one traces \
             rays, else the CPU; gpu, the GPU's ray queries alone, on D3D12 where dxc/dxcompiler.dll stands beside the \
             CLI, else on Vulkan; cpu, the reference, every slot and cover within one step of the GPU's",
          )
          .long("backend")
          .default_value(RayBackend::default().get_name())
          .value_parser(RayBackend::ALL.map(RayBackend::get_name).to_vec()),
      )
      .arg(
        Arg::new("ai-covers")
          .help(
            "Covers the ai step computes: full, traced against build.cform as xrAI -f traces them; none, every cover 0 \
             as xrAI -draft leaves them, needing no build.cform, for a map to walk before the geometry is compiled",
          )
          .long("ai-covers")
          .default_value(AiMapCovers::default().get_name())
          .value_parser(AiMapCovers::ALL.map(AiMapCovers::get_name).to_vec()),
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
      backend: matches
        .get_one::<String>("backend")
        .and_then(|backend| RayBackend::from_name(backend))
        .unwrap_or_default(),
      dxc: Self::find_dxc(),
      gpu: None,
      ai_covers: matches
        .get_one::<String>("ai-covers")
        .and_then(|covers| AiMapCovers::from_name(covers))
        .unwrap_or_default(),
      level,
    };
    let output: OutputOptions = context.get_output().clone();
    let report: LevelCompileReport = Self::compile(&output, &request)?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl CompileCommand {
  /// Where DXC stands beside the CLI's executable, as the app bundles it for the renderer.
  const DXC_LIBRARY: [&'static str; 2] = ["dxc", "dxcompiler.dll"];

  /// DXC's library beside the executable, where it stands there; none otherwise.
  fn find_dxc() -> Option<PathBuf> {
    let executable: PathBuf = std::env::current_exe().ok()?;
    let library: PathBuf = Self::DXC_LIBRARY
      .iter()
      .fold(executable.parent()?.to_path_buf(), |path, part| path.join(part));

    library.is_file().then_some(library)
  }

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
        LevelCompileStepRun::DetailLighter { slots, gpu } => format!(
          "{} slots lit, {} touching no face, {} emptied, {} unplanted, traced on {}",
          slots.lit,
          slots.untouched,
          slots.emptied,
          slots.unplanted,
          gpu.as_deref().unwrap_or("the CPU")
        ),
        LevelCompileStepRun::AiMapCompiler {
          covers,
          gpu,
          guid,
          census,
          graph,
        } => {
          for finding in graph.checks().iter().flat_map(|check| check.findings()) {
            xrf_output::warning!(output, "{}", finding.message());
          }

          format!(
            "{} nodes, {} covers{}, the identity {}; one-way nodes {}, parts {}",
            census.nodes,
            covers.get_name(),
            match (covers, gpu) {
              (AiMapCovers::None, _) => String::new(),
              (AiMapCovers::Full, gpu) => format!(" traced on {}", gpu.as_deref().unwrap_or("the CPU")),
            },
            match guid {
              AiMapGuid::Kept => "kept",
              AiMapGuid::Derived => "derived",
            },
            census.one_way_nodes,
            census.components
          )
        }
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
