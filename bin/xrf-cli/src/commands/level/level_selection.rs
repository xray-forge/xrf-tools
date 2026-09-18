//! The arguments that name one compiled level, shared by every command in the domain.

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayVfs};

use crate::commands::level::level_assets::LevelAssets;

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
    command
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
      .arg(
        Arg::new("source")
          .help("How a root is mounted, which for an installation is the default")
          .long("source")
          .default_value("auto")
          .value_parser([
            "auto",
            "directory",
            "volumes",
            "installation",
            "containing-installation",
          ]),
      )
      .group(ArgGroup::new("selection").args(["path", "root"]).required(true))
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
      source: XrayMountMode::try_from(
        matches
          .get_one::<String>("source")
          .expect("Expected source mode to default")
          .as_str(),
      )?,
    })
  }

  /// Whether the level was named as a directory rather than as one of an installation's.
  pub const fn is_directory(&self) -> bool {
    self.path.is_some()
  }

  /// Mounts the roots this selection names, for a selection that names any.
  ///
  /// Kept apart from [`Self::open`] because the mounted VFS outlives the assets borrowed from it.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted.
  pub fn mount(&self) -> XrfResult<Option<XrayVfs>> {
    match self.path {
      Some(_) => Ok(None),
      None => Ok(Some(LevelAssets::roots_of(&self.roots, self.source).open()?)),
    }
  }

  /// Opens the level this selection names.
  ///
  /// # Errors
  ///
  /// Returns an error when no level was named among the roots, or the named one holds no `level` file.
  pub fn open<'a>(&'a self, vfs: Option<&'a XrayVfs>, scope: &'a XrayLookupScope) -> XrfResult<LevelAssets<'a>> {
    match (&self.path, vfs) {
      (Some(path), _) => Ok(LevelAssets::open_directory(path)),
      (None, Some(vfs)) => {
        let name: &String = self
          .name
          .as_ref()
          .ok_or_else(|| Self::new_name_required_error(vfs, scope))?;

        LevelAssets::open_asset(vfs, scope, name)
      }
      (None, None) => Err(XrfError::new_invalid_error(
        "Expected either a level directory or roots to mount, which the argument group requires",
      )),
    }
  }

  /// The error for roots given without a level, which names what they hold so the next run can pick one.
  fn new_name_required_error(vfs: &XrayVfs, scope: &XrayLookupScope) -> XrfError {
    match LevelAssets::list_names(vfs, scope) {
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
