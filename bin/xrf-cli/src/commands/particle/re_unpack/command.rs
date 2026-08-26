use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::ParticlesFile;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

#[derive(Default)]
pub struct ReUnpackCommand;

impl GenericCommand for ReUnpackCommand {
  fn operation(&self) -> &'static str {
    "re-unpack"
  }

  /// Create command for re-unpack of particle file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to re-unpack provided particle directory into another directory")
      .arg(
        Arg::new("path")
          .help("Path to unpacked particle directory")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting unpacked particle")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Re-unpack provided particle dir and validate it.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let destination: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output path to be provided");

    log::info!("Starting importing particle file {}", path.display());
    log::info!("Re-unpack into {}", destination.display());

    let started_at: Instant = Instant::now();
    let particles_file: Box<ParticlesFile> = Box::new(ParticlesFile::import_from_path(path)?);
    let import_duration: Duration = started_at.elapsed();

    particles_file.export_to_path(destination)?;

    let export_duration: Duration = started_at.elapsed() - import_duration;

    log::info!(
      "Import particle file took: {}",
      xrf_utils::format_duration(import_duration)
    );
    log::info!(
      "Export particle file took: {}",
      xrf_utils::format_duration(export_duration)
    );

    log::info!("Particles file was re-unpacked into {}", destination.display());

    context.set_result(|| FileConversionReport::new(path, destination, import_duration, export_duration))?;

    Ok(())
  }
}
