use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::XRayByteOrder;

const PATH_ARGUMENT: &str = "path";

/// Declares what opening a particle library takes: its file.
pub trait ParticlesFileArguments {
  /// Declares the required `-p/--path` to a `particles.xr`.
  #[must_use]
  fn with_particles_file(self) -> Self;
}

impl ParticlesFileArguments for Command {
  fn with_particles_file(self) -> Self {
    self.arg(
      Arg::new(PATH_ARGUMENT)
        .help("Path to the particle library, particles.xr")
        .short('p')
        .long(PATH_ARGUMENT)
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }
}

/// The library file a command names.
pub fn requested_particles_path(matches: &ArgMatches) -> &PathBuf {
  matches
    .get_one::<PathBuf>(PATH_ARGUMENT)
    .expect("Expected the required particle library path")
}

/// Read the particle library a command names.
///
/// # Errors
///
/// Returns an error when the file cannot be read or is no particle library.
pub fn open_particles_file(matches: &ArgMatches) -> XrfResult<ParticlesFile> {
  ParticlesFile::read_from_path::<XRayByteOrder, _>(requested_particles_path(matches))
}
