use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs, io};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

#[derive(Default)]
pub struct UnpackCommand;

impl GenericCommand for UnpackCommand {
  fn operation(&self) -> &'static str {
    "unpack"
  }

  /// Create command to unpack spawn file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to unpack provided *.spawn into separate files")
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
          .help("Path to folder for exporting")
          .short('d')
          .long("dest")
          .default_value("unpacked")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help("Whether existing unpacked data should be pruned if destination folder exists")
          .short('f')
          .long("force")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Unpack provided *.spawn file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let destination: &PathBuf = matches
      .get_one::<_>("dest")
      .expect("Expected valid output path to be provided");

    let force: bool = matches.get_flag("force");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Starting parsing spawn file: {}", format_path(path));
    xrf_output::info!(output, "Unpack destination: {}", format_path(destination));

    // Apply force flag and delete existing directories.
    if force && destination.exists() && destination.is_dir() {
      fs::remove_dir_all(destination)?;
    }

    // Re-validate that provided output can be used.
    if destination.exists() && destination.is_dir() {
      return Err(
        io::Error::new(
          io::ErrorKind::AlreadyExists,
          "Unpack output directory already exists, use --force to prune destination folder",
        )
        .into(),
      );
    }

    let started_at: Instant = Instant::now();
    let spawn_file: Box<SpawnFile> = Box::new(SpawnFile::read_from_path::<XRayByteOrder, _>(path)?);
    let read_duration: Duration = started_at.elapsed();

    spawn_file.export_to_path::<XRayByteOrder, _>(destination)?;

    let unpack_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::info!(
      output,
      "Read spawn file took: {}",
      xrf_utils::format_duration(read_duration)
    );
    xrf_output::info!(
      output,
      "Export spawn file took: {}",
      xrf_utils::format_duration(unpack_duration)
    );

    context.set_result(|| FileConversionReport::new(path, destination, read_duration, unpack_duration))?;

    Ok(())
  }
}
