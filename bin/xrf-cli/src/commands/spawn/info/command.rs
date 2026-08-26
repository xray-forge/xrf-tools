use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_output::OutputOptions;

use super::report::SpawnInfoReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  /// Create command for printing spawn file info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about provided spawn file")
      .arg(
        Arg::new("path")
          .help("Path to spawn file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Print information about spawn file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read spawn file {}", path.display());

    let spawn_file: Box<SpawnFile> = Box::new(SpawnFile::read_from_path::<XRayByteOrder, _>(path)?);

    xrf_output::info!(output, "Spawn file information:");

    xrf_output::info!(output, "Version: {}", spawn_file.header.version);
    xrf_output::info!(output, "GUID: {}", spawn_file.header.guid);
    xrf_output::info!(output, "Levels count: {}", spawn_file.header.levels_count);
    xrf_output::info!(output, "Objects count: {}", spawn_file.header.objects_count);

    xrf_output::info!(
      output,
      "Artefact spawn points: {}",
      spawn_file.artefact_spawn.nodes.len()
    );

    xrf_output::info!(output, "Patrols: {}", spawn_file.patrols.patrols.len());

    xrf_output::info!(output, "Level version: {}", spawn_file.graphs.header.version);
    xrf_output::info!(
      output,
      "Level graph vertices: {}",
      spawn_file.graphs.header.vertices_count
    );
    xrf_output::info!(output, "Level graph points: {}", spawn_file.graphs.header.points_count);
    xrf_output::info!(output, "Level graph edges: {}", spawn_file.graphs.header.edges_count);

    context.set_result(|| SpawnInfoReport::new(&spawn_file))?;

    Ok(())
  }
}
