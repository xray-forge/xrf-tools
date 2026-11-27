use xrf_error::XrfResult;

use crate::file::config_file::ConfigFile;

/// Where the game reads its tree's configs from, kept for the game's start (plan decision 25): `system.ltx`, resolved
/// once, and any other config by its path under `configs`, resolved the same way, as X-Ray reads `pSettings` and the
/// scripts' `ini_file` reads one.
pub trait ConfigSource {
  /// `configs\system.ltx`.
  fn get_system(&self) -> &ConfigFile;

  /// The config at `path` under `configs`, such as `alife.ltx` or `misc\simulation.ltx`.
  ///
  /// # Errors
  ///
  /// Returns why it cannot be read: the tree has none, or it does not resolve.
  fn read_file(&self, path: &str) -> XrfResult<ConfigFile>;
}
