use std::path::Path;

use xrf_error::XrfResult;
use xrf_vfs::{XrayCachePolicy, XrayMountMode, XraySearch, XraySearchPlan, XrayVfs};

use crate::core::command::CommandInput;

/// How every LTX command tells a game installation from a folder of configs, and mounts one.
pub(crate) struct LtxInstallation;

impl LtxInstallation {
  /// The input every LTX command reading an installation states.
  pub(crate) fn input() -> CommandInput {
    CommandInput::path(concat!(
      "Path to a folder with ltx files, or to a game installation root holding fsgame.ltx ",
      "(fsgame_<game>.ltx in an Enhanced Edition)"
    ))
  }

  /// A search over a game installation's declared sources, or `None` when `path` is not one.
  ///
  /// The LTX commands treat only a path directly holding an fsgame file as an installation, and share the rule from
  /// here so they cannot drift. A named directory stays a directory: widening `<install>\gamedata\configs` to the
  /// whole game would read or rewrite thousands of configs nobody asked about.
  ///
  /// # Errors
  ///
  /// Returns an error when an fsgame file is present but cannot be read, decoded, or parsed, or when a declared source
  /// cannot be mounted.
  pub(crate) fn mount(path: &Path) -> XrfResult<Option<XraySearch>> {
    if !XrayMountMode::declares_installation(path) {
      return Ok(None);
    }

    Ok(Some(
      XraySearchPlan::new()
        .with_root_mode(XraySearchPlan::INSTALLATION_STEP, path, XrayMountMode::Installation)?
        .open_in(XrayVfs::new().with_cache_policy(XrayCachePolicy::configs()))?,
    ))
  }
}
