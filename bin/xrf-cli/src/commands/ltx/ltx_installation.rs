use std::path::Path;

use xrf_error::XrfResult;
use xrf_vfs::{XrayCachePolicy, XrayMountMode, XrayVfs};

/// What `--path` accepts, for every LTX command that reads an installation.
pub const INSTALLATION_PATH_HELP: &str = concat!(
  "Path to a folder with ltx files, or to a game installation root holding fsgame.ltx ",
  "(fsgame_<game>.ltx in an Enhanced Edition)"
);

/// Mounts a game installation's declared sources, or returns `None` when `path` is not one.
///
/// Both LTX commands treat only a path directly holding an fsgame file as an installation, and share the rule from here
/// so they cannot drift. A named directory stays a directory: widening `--path <install>\gamedata\configs` to the whole
/// game would read or rewrite thousands of configs nobody asked about.
///
/// # Errors
///
/// Returns an error when an fsgame file is present but cannot be read, decoded, or parsed, or when a declared source
/// cannot be mounted.
pub fn mount_installation(path: &Path) -> XrfResult<Option<XrayVfs>> {
  if !XrayMountMode::declares_installation(path) {
    return Ok(None);
  }

  Ok(Some(
    XrayVfs::open(XrayMountMode::Installation, path)?.with_cache_policy(XrayCachePolicy::configs()),
  ))
}
