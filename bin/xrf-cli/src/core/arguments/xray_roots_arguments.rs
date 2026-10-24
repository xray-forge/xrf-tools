use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_vfs::{XrayMountMode, XrayRoots};

/// Repeatable `--path`, `--source` and `--prefix`: the roots a command reads, in one vocabulary, so repeating `--path`
/// layers a tree in front of an installation exactly as the desktop app does it, whichever command is asked.
pub struct XrayRootsArguments;

impl XrayRootsArguments {
  const PATH: &'static str = "path";
  const SOURCE: &'static str = "source";
  const PREFIX: &'static str = "prefix";

  /// How each mount mode reads a path, for the help of `--source`.
  const SOURCE_HELP: &'static str = "How to read the path: auto treats it as an installation only when it declares \
                                     one, directory ignores any declaration, volumes mounts every archive volume \
                                     beneath it, installation requires one, containing-installation searches parent \
                                     directories for one";

  /// Declares the three on `command`: `path_help` says what a path holds, `prefix_help` what a prefix limits it to.
  /// The paths are read as their containing installation unless `--source` says otherwise.
  pub fn declare(command: Command, path_help: &'static str, prefix_help: &'static str) -> Command {
    let command: Command = command.arg(
      Arg::new(Self::PATH)
        .help(path_help)
        .short('p')
        .long(Self::PATH)
        .required(true)
        // Both spellings layer: repeat the flag, or list several values after one of them.
        .action(ArgAction::Append)
        .num_args(1..)
        .value_parser(value_parser!(PathBuf)),
    );

    Self::declare_source(command, XrayMountMode::ContainingInstallation).arg(
      Arg::new(Self::PREFIX)
        .help(prefix_help)
        .long(Self::PREFIX)
        .value_parser(value_parser!(String)),
    )
  }

  /// Declares `--source` alone, defaulting to `default`, for a command that names its roots its own way.
  pub fn declare_source(command: Command, default: XrayMountMode) -> Command {
    command.arg(
      Arg::new(Self::SOURCE)
        .help(Self::SOURCE_HELP)
        .long(Self::SOURCE)
        .default_value(default.as_str())
        .value_parser(XrayMountMode::NAMES),
    )
  }

  /// The roots the command was asked to read, in the order they layer.
  ///
  /// # Errors
  ///
  /// Returns an error when `--source` names no mount mode.
  pub fn of(matches: &ArgMatches) -> XrfResult<XrayRoots> {
    Ok(XrayRoots::from_paths(
      Self::list_paths(matches).into_iter().cloned(),
      Self::source_of(matches)?,
    ))
  }

  /// The paths as given, before any is mounted, for a command that reads one file directly.
  pub fn list_paths(matches: &ArgMatches) -> Vec<&PathBuf> {
    matches
      .get_many::<PathBuf>(Self::PATH)
      .expect("Expected at least one path to be provided")
      .collect()
  }

  /// How the command was asked to mount each of its paths.
  ///
  /// # Errors
  ///
  /// Returns an error when `--source` names no mount mode.
  pub fn source_of(matches: &ArgMatches) -> XrfResult<XrayMountMode> {
    XrayMountMode::try_from(
      matches
        .get_one::<String>(Self::SOURCE)
        .expect("Expected source mode to default")
        .as_str(),
    )
  }

  /// The logical subtree the command was limited to, if any.
  pub fn prefix_of(matches: &ArgMatches) -> Option<&str> {
    matches.get_one::<String>(Self::PREFIX).map(String::as_str)
  }
}
