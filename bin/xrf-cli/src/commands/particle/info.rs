use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{ParticlesFile, XRayByteOrder};
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  /// Create command for printing particle file info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about provided particle file")
      .arg(
        Arg::new("path")
          .help("Path to particle file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  /// Print information about particle file.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    xrf_output::info!(output, "Read particle file {}", path.display());

    let particles_file: Box<ParticlesFile> = Box::new(ParticlesFile::read_from_path::<XRayByteOrder, _>(path)?);

    xrf_output::info!(output, "Particles file information:");

    xrf_output::info!(output, "Version: {}", particles_file.header.version);
    xrf_output::info!(output, "Effects count: {}", particles_file.effects.effects.len());
    xrf_output::info!(output, "Groups count: {}", particles_file.groups.groups.len());

    Ok(())
  }
}
