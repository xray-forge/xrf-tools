use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};

/// The named arguments a packed-file command writing a file takes: what it writes (`-d`) and whether it may replace
/// that (`-f`). What it reads is its positional input.
pub(crate) struct PackedFileArguments;

impl PackedFileArguments {
  const DEST: &'static str = "dest";
  const FORCE: &'static str = "force";

  /// Declares a required `--dest`, described by `dest_help`, on `command`.
  pub(crate) fn declare_dest(command: Command, dest_help: String) -> Command {
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
  pub(crate) fn declare_force(command: Command) -> Command {
    command.arg(
      Arg::new(Self::FORCE)
        .help("Replace the destination when it is already there, once the new one is ready")
        .short('f')
        .long(Self::FORCE)
        .action(ArgAction::SetTrue),
    )
  }

  /// What the command writes.
  pub(crate) fn dest_of(matches: &ArgMatches) -> &PathBuf {
    matches
      .get_one::<PathBuf>(Self::DEST)
      .expect("Expected a destination to be provided")
  }

  /// Whether the command may replace its destination.
  pub(crate) fn is_forced(matches: &ArgMatches) -> bool {
    matches.try_get_one::<bool>(Self::FORCE).ok().flatten() == Some(&true)
  }
}
