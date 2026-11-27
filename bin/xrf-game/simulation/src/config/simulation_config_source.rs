use xrf_error::XrfResult;
use xrf_ltx::Ltx;

/// Where the simulation reads its configs from: the tree's `system.ltx`, resolved with its includes and inheritance,
/// and any other config by its path under `configs`, resolved the same way, as the scripts' `ini_file` reads one.
pub trait SimulationConfigSource {
  /// `configs\system.ltx`, which holds the squad and character sections.
  fn get_system(&self) -> &Ltx;

  /// The config at `path` under `configs`, such as `misc\simulation.ltx`.
  ///
  /// # Errors
  ///
  /// Returns why it cannot be read: the tree has none, or it does not resolve.
  fn read_config(&self, path: &str) -> XrfResult<Ltx>;
}
