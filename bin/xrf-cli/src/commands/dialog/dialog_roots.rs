use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

const PATH_ARGUMENT: &str = "path";
const SOURCE_ARGUMENT: &str = "source";
const PREFIX_ARGUMENT: &str = "prefix";

/// Declares the roots every dialog command reads, in the builder style clap itself uses.
///
/// One vocabulary for naming roots, so repeating `--path` layers a tree in front of an installation exactly as the
/// desktop app does it, whichever dialog command is asked.
pub trait DialogRootsArguments {
  /// Declares repeatable `--path`, `--source` and `--prefix`.
  #[must_use]
  fn with_dialog_roots(self) -> Self;
}

impl DialogRootsArguments for Command {
  fn with_dialog_roots(self) -> Self {
    self
      .arg(
        Arg::new(PATH_ARGUMENT)
          .help("Root holding dialog xml. Repeat to layer roots, highest priority first")
          .short('p')
          .long(PATH_ARGUMENT)
          .required(true)
          // Both spellings layer: repeat the flag, or list several values after one of them.
          .action(ArgAction::Append)
          .num_args(1..)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new(SOURCE_ARGUMENT)
          .help(
            "How to read the path: auto treats it as an installation only when it declares one, directory ignores any declaration, volumes mounts every archive volume beneath it, installation requires one, containing-installation searches parent directories for one",
          )
          .long(SOURCE_ARGUMENT)
          .default_value("containing-installation")
          .value_parser(["auto", "directory", "volumes", "installation", "containing-installation"]),
      )
      .arg(
        Arg::new(PREFIX_ARGUMENT)
          .help("Limit to one logical subtree, such as configs\\gameplay")
          .long(PREFIX_ARGUMENT)
          .value_parser(value_parser!(String)),
      )
  }
}

/// How a dialog command was asked to mount each of its paths.
///
/// # Errors
///
/// Returns an error when `--source` names no mount mode.
pub fn requested_dialog_source(matches: &ArgMatches) -> XrfResult<XrayMountMode> {
  XrayMountMode::try_from(
    matches
      .get_one::<String>(SOURCE_ARGUMENT)
      .expect("Expected source mode to default")
      .as_str(),
  )
}

/// The roots a dialog command was asked to read, in the order they layer.
///
/// # Errors
///
/// Returns an error when `--source` names no mount mode.
pub fn requested_dialog_roots(matches: &ArgMatches) -> XrfResult<XrayRoots> {
  let source: XrayMountMode = requested_dialog_source(matches)?;

  Ok(XrayRoots::new(
    matches
      .get_many::<PathBuf>(PATH_ARGUMENT)
      .expect("Expected at least one path to be provided")
      .map(|path| XrayRoot::new(path.clone(), source)),
  ))
}

/// The logical subtree a dialog command was limited to, if any.
pub fn requested_dialog_prefix(matches: &ArgMatches) -> Option<&str> {
  matches.get_one::<String>(PREFIX_ARGUMENT).map(String::as_str)
}
