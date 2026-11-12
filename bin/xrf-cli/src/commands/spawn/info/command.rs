use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::SpawnInfoReport;
use super::spawn_info_file::SpawnInfoFile;
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

    let file: SpawnInfoFile = SpawnInfoFile::read_from_path(path)?;

    file.print(&output);

    context.set_result(|| SpawnInfoReport::new(&file))?;

    Ok(())
  }
}
