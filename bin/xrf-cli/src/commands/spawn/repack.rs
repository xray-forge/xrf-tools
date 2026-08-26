use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{SpawnFile, XRayByteOrder};

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct RepackCommand;

impl GenericCommand for RepackCommand {
  fn operation(&self) -> &'static str {
    "repack"
  }

  /// Create command for repack of spawn file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to repack provided *.spawn into another file")
      .arg(
        Arg::new("path")
          .help("Path to *.spawn file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting *.spawn file")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Repack provided *.spawn file and validate it.
  fn execute(&self, matches: &ArgMatches, _context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid input path to be provided");

    let destination: &PathBuf = matches
      .get_one::<_>("dest")
      .expect("Expected valid output path to be provided");

    log::info!("Starting parsing spawn file {}", path.display());
    log::info!("Repack into {}", destination.display());

    let started_at: Instant = Instant::now();
    let spawn_file: Box<SpawnFile> = Box::new(SpawnFile::read_from_path::<XRayByteOrder, _>(path)?);
    let read_duration: Duration = started_at.elapsed();

    spawn_file.write_to_path::<XRayByteOrder, _>(destination)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    log::info!("Read spawn file took: {}", xrf_utils::format_duration(read_duration));
    log::info!("Write spawn file took: {}", xrf_utils::format_duration(write_duration));

    log::info!("Spawn file was repacked into {}", destination.display());

    Ok(())
  }
}
