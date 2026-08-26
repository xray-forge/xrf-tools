use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs, io};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{SpawnFile, XRayByteOrder};

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct PackCommand;

impl GenericCommand for PackCommand {
  fn operation(&self) -> &'static str {
    "pack"
  }

  /// Create command packing of spawn file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack unpacked spawn files into single *.spawn")
      .arg(
        Arg::new("path")
          .help("Path to unpacked spawn file folder")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting packed *.spawn file")
          .short('d')
          .long("dest")
          .default_value("unpacked")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help("Whether existing packed spawn should be pruned if destination folder exists")
          .short('f')
          .long("force")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Pack *.spawn file based on provided arguments.
  fn execute(&self, matches: &ArgMatches, _context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let destination: &PathBuf = matches
      .get_one::<_>("dest")
      .expect("Expected valid output path to be provided");

    let force: bool = matches.get_flag("force");

    log::info!("Starting packing spawn file {}", path.display());
    log::info!("Pack destination {}", destination.display());

    // Apply force flag and delete existing spawn output.
    if force && destination.exists() && destination.is_file() {
      fs::remove_file(destination)?;
    }

    // Re-validate that provided output can be used.
    if destination.exists() && destination.is_file() {
      return Err(
        io::Error::new(
          io::ErrorKind::AlreadyExists,
          "Pack output file already exists, use --force to prune destination",
        )
        .into(),
      );
    }

    let started_at: Instant = Instant::now();
    let spawn_file: Box<SpawnFile> = Box::new(SpawnFile::import_from_path::<XRayByteOrder, _>(path)?);
    let read_duration: Duration = started_at.elapsed();

    spawn_file.write_to_path::<XRayByteOrder, _>(destination)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    log::info!("Read spawn file took: {}", xrf_utils::format_duration(read_duration));
    log::info!(
      "Writing packed spawn file took: {}",
      xrf_utils::format_duration(write_duration)
    );

    Ok(())
  }
}
