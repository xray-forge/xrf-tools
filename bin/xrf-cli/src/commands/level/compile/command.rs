use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_level_compiler::{
  AiMapCovers, AiMapGuid, LevelCompileReport, LevelCompileRequest, LevelCompileStep, LevelCompileStepRun,
  LevelCompiler, LightQuality, RayBackend,
};
use xrf_output::OutputOptions;

use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Compiles a level directory `level build` wrote, step by step, each by our own compiler: the geometry, its light
/// (vertices, lightmaps and terrain), the details' light and the AI map.
#[derive(Default)]
pub(crate) struct CompileCommand;

impl GenericCommand for CompileCommand {
  fn operation(&self) -> &'static str {
    "compile"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Compile a level directory that level build wrote into the files the game loads. The steps run in this \
         process, in order: geometry (level, level.geom, level.geomx and level.cform, as xrLC builds them, and the \
         build.cform and build.lights the later steps read), light (the light of the vertices, lightmaps and terrain, baked in \
         the same pass as the geometry, so it runs only with geometry; without it the level gets flat neutral \
         light), details (level.details lit from build.details, as xrDO_Light lights it) and ai (level.ai from \
         build.aimap, as xrAI -f compiles it, then checked as a graph). Before any step runs, the compile checks \
         that every input of the chosen steps is there, and a .thm for every texture build.prj names",
      )
      .arg(
        Arg::new("path")
          .help("Path to the level directory, <levels>/<level>, that level build wrote into")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("steps")
          .help(
            "Steps to run, comma-separated; they always run in the order geometry, light, details, ai. Every step by \
             default; light runs only with geometry",
          )
          .long("steps")
          .value_delimiter(',')
          .value_parser(LevelCompileStep::ALL.map(LevelCompileStep::get_name).to_vec()),
      )
      .arg(
        Arg::new("gamedata")
          .help(
            "Game data directory to read gamemtl.xr and shaders_xrlc.xr from; by default the gamedata directory \
             holding the level's levels directory",
          )
          .long("gamedata")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("textures")
          .help(
            "Textures root holding a .thm for every texture build.prj names, and the image of each texture with \
             alpha; repeatable, each file taken from the first root holding it; by default the game data's textures \
             directory",
          )
          .long("textures")
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help(
            "Overwrite level.details and level.ai where the level directory already holds them. A replaced level.ai \
             keeps its identity while its graph is unchanged, so the level's spawn still loads",
          )
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("backend")
          .help(
            "What traces the rays of the light, details and ai steps: auto uses the GPU where it supports ray \
             tracing, else the CPU; gpu uses the GPU only, on D3D12 where dxc/dxcompiler.dll is beside the CLI, else \
             on Vulkan; cpu uses the CPU, the reference result",
          )
          .long("backend")
          .default_value(RayBackend::default().get_name())
          .value_parser(RayBackend::ALL.map(RayBackend::get_name).to_vec()),
      )
      .arg(
        Arg::new("ai-covers")
          .help(
            "Covers the ai step computes: full traces them against build.cform, as xrAI -f does; none sets every \
             cover to 0, as xrAI -draft does, and needs no build.cform, for a map to walk before the geometry is \
             compiled",
          )
          .long("ai-covers")
          .default_value(AiMapCovers::default().get_name())
          .value_parser(AiMapCovers::ALL.map(AiMapCovers::get_name).to_vec()),
      )
      .arg(
        Arg::new("quality")
          .help(
            "How finely the light step lights the level: scene uses build.prj's own sky, sun, lightmap density and \
             samples; draft is a quick estimate, regenerating the sky and sun (also those written to build.lights) at \
             quality 1, lighting lightmaps at 3 texels a metre with one sample each, and each multi-use object, such \
             as a tree, from one sample",
          )
          .long("quality")
          .default_value(LightQuality::default().get_name())
          .value_parser(LightQuality::ALL.map(LightQuality::get_name).to_vec()),
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
      textures,
      gamedata,
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
      light_quality: matches
        .get_one::<String>("quality")
        .and_then(|quality| LightQuality::from_name(quality))
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
        LevelCompileStepRun::GeometryCompiler { census } => format!(
          "{} static visuals of {} triangles, {} trees, {} visuals progressive, {} sectors",
          census.visuals, census.triangles, census.trees, census.progressive, census.sectors,
        ),
        LevelCompileStepRun::LightBaker { census, gpu } => format!(
          "vertices and trees lit, {} lightmap charts on {} pages, {} textures lit implicitly, traced on {}",
          census.charts,
          census.lightmaps,
          census.implicit_maps,
          gpu.as_deref().unwrap_or("the CPU")
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
      "Compiled {} ({})",
      report.level,
      report
        .steps
        .iter()
        .map(|step| step.step.get_name())
        .collect::<Vec<_>>()
        .join(", ")
    );

    Ok(report)
  }
}
