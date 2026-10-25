use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_gamemtl::GameMtlFile;

/// `-p/--path` to a `gamemtl.xr`: what opening a game material library takes.
pub struct GameMtlFileArguments;

impl GameMtlFileArguments {
  const PATH: &'static str = "path";

  /// Declares the required library path on `command`.
  pub fn declare(command: Command) -> Command {
    command.arg(
      Arg::new(Self::PATH)
        .help("Path to the game material library, gamemtl.xr")
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
      .expect("Expected the required game material library path")
  }

  /// Reads the game material library a command names.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is no game material library the engine loads.
  pub fn open(matches: &ArgMatches) -> XrfResult<GameMtlFile> {
    GameMtlFile::read_from_path::<XRayByteOrder, _>(Self::path_of(matches))
  }
}
