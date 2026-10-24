use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};

/// The arguments every packed-file command reads: what it reads (`-p`), what it writes (`-d`), and whether it may
/// replace that (`-f`).
pub struct PackedFileArguments;

impl PackedFileArguments {
  const PATH: &'static str = "path";
  const DEST: &'static str = "dest";
  const FORCE: &'static str = "force";

  /// Declares `--path`, described by `path_help`, on `command`.
  pub fn declare_path(command: Command, path_help: String) -> Command {
    command.arg(
      Arg::new(Self::PATH)
        .help(path_help)
        .short('p')
        .long(Self::PATH)
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  /// Declares a required `--dest`, described by `dest_help`, on `command`.
  pub fn declare_dest(command: Command, dest_help: String) -> Command {
    command.arg(
      Arg::new(Self::DEST)
        .help(dest_help)
        .short('d')
        .long(Self::DEST)
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  /// Declares `--force`, letting the command replace a `--dest` that is already there, on `command`.
  pub fn declare_force(command: Command) -> Command {
    command.arg(
      Arg::new(Self::FORCE)
        .help("Replace the destination when it is already there, once the new one is ready")
        .short('f')
        .long(Self::FORCE)
        .action(ArgAction::SetTrue),
    )
  }

  /// What the command reads.
  pub fn path_of(matches: &ArgMatches) -> &PathBuf {
    matches
      .get_one::<PathBuf>(Self::PATH)
      .expect("Expected a path to be provided")
  }

  /// What the command writes.
  pub fn dest_of(matches: &ArgMatches) -> &PathBuf {
    matches
      .get_one::<PathBuf>(Self::DEST)
      .expect("Expected a destination to be provided")
  }

  /// Whether the command may replace its destination.
  pub fn is_forced(matches: &ArgMatches) -> bool {
    matches.try_get_one::<bool>(Self::FORCE).ok().flatten() == Some(&true)
  }
}
