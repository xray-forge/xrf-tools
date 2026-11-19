use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::XRayByteOrder;

use clap::ArgMatches;

use crate::core::arguments::InputArgument;
use crate::core::command::CommandInput;

/// A `particles.xr` as a command's input: what opening a particle library takes.
pub(crate) struct ParticlesFileInput;

impl ParticlesFileInput {
  /// The input every command reading one particle library states.
  pub(crate) fn input() -> CommandInput {
    CommandInput::path("Path to the particle library, particles.xr")
  }

  /// Reads the particle library a command names.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is no particle library.
  pub(crate) fn open(matches: &ArgMatches) -> XrfResult<ParticlesFile> {
    ParticlesFile::read_from_path::<XRayByteOrder, _>(InputArgument::of(matches))
  }
}
