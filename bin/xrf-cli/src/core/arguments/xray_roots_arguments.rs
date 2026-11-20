use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::core::arguments::InputArgument;

/// The roots a command reads, as its positional input, with `--source` and `--prefix`: one vocabulary, so a second root
/// layers a tree in front of an installation exactly as the desktop app does it, whichever command is asked.
pub(crate) struct XrayRootsArguments;

impl XrayRootsArguments {
  const SOURCE: &'static str = "source";
  const PREFIX: &'static str = "prefix";

  /// How each mount mode reads a path, for the help of `--source`.
  const SOURCE_HELP: &'static str = "How to read the path: auto treats it as an installation only when it declares \
                                     one, directory ignores any declaration, volumes mounts every archive volume \
                                     beneath it, installation requires one, containing-installation searches parent \
                                     directories for one";

  /// Declares the roots and both flags on `command`: `path_help` says what a root holds, `prefix_help` what a prefix
  /// limits it to. The roots are read as their containing installation unless `--source` says otherwise.
  pub(crate) fn declare(command: Command, path_help: String, prefix_help: String) -> Command {
    let command: Command = InputArgument::declare_many(command, path_help);

    Self::declare_source(command, XrayMountMode::ContainingInstallation).arg(
      Arg::new(Self::PREFIX)
        .help(prefix_help)
        .long(Self::PREFIX)
        .value_parser(value_parser!(String)),
    )
  }

  /// Declares `--source` alone, defaulting to `default`, for a command that names its roots its own way.
  pub(crate) fn declare_source(command: Command, default: XrayMountMode) -> Command {
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
  pub(crate) fn of(matches: &ArgMatches) -> XrfResult<XrayRoots> {
    Ok(XrayRoots::from_paths(
      InputArgument::list_of(matches).into_iter().cloned(),
      Self::source_of(matches)?,
    ))
  }

  /// The one source file the command was handed directly, when its only path names a file.
  ///
  /// A VFS mounts a directory, never a file, so a command that also reads a single source keeps that convenience
  /// through this rather than through [`Self::of`]: one file and no roots is the case it exists for.
  pub(crate) fn single_file_of(matches: &ArgMatches) -> Option<&PathBuf> {
    match InputArgument::list_of(matches).as_slice() {
      [path] if path.is_file() => Some(path),
      _ => None,
    }
  }

  /// How the command was asked to mount each of its paths.
  ///
  /// # Errors
  ///
  /// Returns an error when `--source` names no mount mode.
  pub(crate) fn source_of(matches: &ArgMatches) -> XrfResult<XrayMountMode> {
    XrayMountMode::try_from(
      matches
        .get_one::<String>(Self::SOURCE)
        .expect("Expected source mode to default")
        .as_str(),
    )
  }

  /// The logical subtree the command was limited to, if any.
  pub(crate) fn prefix_of(matches: &ArgMatches) -> Option<&str> {
    matches.get_one::<String>(Self::PREFIX).map(String::as_str)
  }
}
