use clap::ArgMatches;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_gamemtl::GameMtlFile;

use crate::core::arguments::InputArgument;
use crate::core::command::CommandInput;

/// A `gamemtl.xr` as a command's input: what opening a game material library takes.
pub(crate) struct GameMtlFileInput;

impl GameMtlFileInput {
  /// The input every command reading one game material library states.
  pub(crate) fn input() -> CommandInput {
    CommandInput::path("Path to the game material library, gamemtl.xr")
  }

  /// Reads the game material library a command names.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is no game material library the engine loads.
  pub(crate) fn open(matches: &ArgMatches) -> XrfResult<GameMtlFile> {
    GameMtlFile::read_from_path::<XRayByteOrder, _>(InputArgument::of(matches))
  }
}
