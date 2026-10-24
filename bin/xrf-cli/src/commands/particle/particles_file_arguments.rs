use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::XRayByteOrder;

/// `-p/--path` to a `particles.xr`: what opening a particle library takes.
pub struct ParticlesFileArguments;

impl ParticlesFileArguments {
  const PATH: &'static str = "path";

  /// Declares the required library path on `command`.
  pub fn declare(command: Command) -> Command {
    command.arg(
      Arg::new(Self::PATH)
        .help("Path to the particle library, particles.xr")
        .short('p')
        .long(Self::PATH)
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  /// The library file a command names.
  pub fn path_of(matches: &ArgMatches) -> &PathBuf {
    matches
      .get_one::<PathBuf>(Self::PATH)
      .expect("Expected the required particle library path")
  }

  /// Reads the particle library a command names.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is no particle library.
  pub fn open(matches: &ArgMatches) -> XrfResult<ParticlesFile> {
    ParticlesFile::read_from_path::<XRayByteOrder, _>(Self::path_of(matches))
  }
}
