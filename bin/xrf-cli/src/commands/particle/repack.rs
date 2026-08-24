use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{ParticlesFile, XRayByteOrder};

use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct RepackCommand;

impl GenericCommand for RepackCommand {
  fn operation(&self) -> &'static str {
    "repack"
  }

  /// Create command for repack of particle file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to repack provided particle.xr into another file")
      .arg(
        Arg::new("path")
          .help("Path to particle file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting particle file")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Repack provided particle file and validate it.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid input path to be provided");

    let destination: &PathBuf = matches
      .get_one::<_>("dest")
      .expect("Expected valid output path to be provided");

    log::info!("Starting parsing particle file {}", path.display());
    log::info!("Repack into {}", destination.display());

    let started_at: Instant = Instant::now();
    let particles_file: Box<ParticlesFile> = Box::new(ParticlesFile::read_from_path::<XRayByteOrder, _>(path)?);
    let read_duration: Duration = started_at.elapsed();

    particles_file.write_to_path::<XRayByteOrder, _>(destination)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    log::info!("Read particle file took: {}", xrf_utils::format_duration(read_duration));
    log::info!(
      "Write particle file took: {}",
      xrf_utils::format_duration(write_duration)
    );

    log::info!("Particles file was repacked into {}", destination.display());

    Ok(())
  }
}
