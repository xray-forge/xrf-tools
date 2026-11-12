use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_spawn::{AnySpawnFile, LevelSpawnFile, SpawnFile, XRayByteOrder};
use xrf_utils::format_path;

use super::report::SpawnInfoReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Print what a spawn file holds").arg(
      Arg::new("path")
        .help("Path to a root spawn such as all.spawn, or to a level's level.spawn")
        .short('p')
        .long("path")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read spawn file {}", format_path(path));

    let file: AnySpawnFile = AnySpawnFile::read_from_path::<XRayByteOrder, _>(path)?;

    match &file {
      AnySpawnFile::Root(file) => Self::print_root(&output, file),
      AnySpawnFile::Level(file) => Self::print_level(&output, file),
    }

    context.set_result(|| SpawnInfoReport::new(&file))?;

    Ok(())
  }
}

impl InfoCommand {
  fn print_root(output: &OutputOptions, file: &SpawnFile) {
    xrf_output::info!(output, "Spawn file information:");
    xrf_output::info!(output, "Version: {}", file.header.version);
    xrf_output::info!(output, "GUID: {}", file.header.guid);
    xrf_output::info!(output, "Levels count: {}", file.header.levels_count);
    xrf_output::info!(output, "Objects count: {}", file.header.objects_count);
    xrf_output::info!(output, "Artefact spawn points: {}", file.artefact_spawn.nodes.len());
    xrf_output::info!(output, "Patrols: {}", file.patrols.patrols.len());
    xrf_output::info!(output, "Level version: {}", file.graphs.header.version);
    xrf_output::info!(output, "Level graph vertices: {}", file.graphs.header.vertices_count);
    xrf_output::info!(output, "Level graph points: {}", file.graphs.header.points_count);
    xrf_output::info!(output, "Level graph edges: {}", file.graphs.header.edges_count);
  }

  fn print_level(output: &OutputOptions, file: &LevelSpawnFile) {
    let sections: BTreeMap<&str, usize> = file.get_sections();

    xrf_output::info!(output, "Level spawn list information:");
    xrf_output::info!(output, "Objects count: {}", file.objects.len());
    xrf_output::info!(output, "Sections count: {}", sections.len());

    for (section, objects) in sections {
      xrf_output::info!(output, "Section '{}' objects: {}", section, objects);
    }
  }
}
