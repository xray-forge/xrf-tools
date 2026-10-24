//! The arguments that name one compiled level, shared by every command in the domain.

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayMountMode, XrayRoots, XraySearch};

use crate::commands::level::level_assets::LevelAssets;
use crate::core::arguments::XrayRootsArguments;

/// A level named either as a directory on disk or as one of an installation's.
pub struct LevelSelection {
  path: Option<PathBuf>,
  roots: Vec<PathBuf>,
  source: XrayMountMode,
  name: Option<String>,
}

impl LevelSelection {
  /// Declares the arguments on a command of the domain, so every one of them names a level the same way.
  pub fn declare(command: Command) -> Command {
    let command: Command = command
      .arg(
        Arg::new("path")
          .help("Path to a compiled level directory, the one holding `level` and `level.geom`")
          .short('p')
          .long("path")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("root")
          .help("Root holding an installation's levels. Repeat to layer roots, highest priority first")
          .short('r')
          .long("root")
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("level")
          .help("Name of the level to read from the mounted roots, as `levels` knows it")
          .short('l')
          .long("level")
          .requires("root")
          .value_parser(value_parser!(String)),
      )
      .group(ArgGroup::new("selection").args(["path", "root"]).required(true));

    // A level's roots are read in auto mode by default, so an installation brings its archives and a bare tree is
    // read as it is.
    XrayRootsArguments::declare_source(command, XrayMountMode::Auto)
  }

  /// Takes the selection out of parsed arguments.
  ///
  /// # Errors
  ///
  /// Returns an error when the source mode is not one a root can be mounted in.
  pub fn of(matches: &ArgMatches) -> XrfResult<Self> {
    Ok(Self {
      name: matches.get_one::<String>("level").cloned(),
      path: matches.get_one::<PathBuf>("path").cloned(),
      roots: matches
        .get_many::<PathBuf>("root")
        .map(|values| values.cloned().collect())
        .unwrap_or_default(),
      source: XrayRootsArguments::source_of(matches)?,
    })
  }

  /// Opens the level this selection names, mounting the roots it names where it names a level among them.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, no level was named among them, or the named one holds no
  /// `level` file.
  pub fn open(&self) -> XrfResult<LevelAssets> {
    if let Some(path) = &self.path {
      return Ok(LevelAssets::open_directory(path));
    }

    let search: XraySearch = XrayRoots::from_paths(self.roots.iter().cloned(), self.source)
      .to_search_plan()?
      .open()?;
    let name: &String = self
      .name
      .as_ref()
      .ok_or_else(|| Self::new_name_required_error(&search))?;

    LevelAssets::open_asset(search, name)
  }

  /// The error for roots given without a level, which names what they hold so the next run can pick one.
  fn new_name_required_error(search: &XraySearch) -> XrfError {
    match LevelAssets::list_names(search) {
      Ok(names) if names.is_empty() => {
        XrfError::new_not_found_error("The mounted roots hold no levels, so none can be named with --level")
      }
      Ok(names) => XrfError::new_invalid_error(format!(
        "Expected --level naming one of the {} the mounted roots hold: {}",
        names.len(),
        names.join(", ")
      )),
      Err(error) => error,
    }
  }
}
