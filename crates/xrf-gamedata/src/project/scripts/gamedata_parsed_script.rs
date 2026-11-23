use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_lua::XRayLuaScript;

/// One runtime script as the checks reading its contents meet it: its logical path, and what parsing it came to, or why
/// it did not read, which the scripts check reports.
pub(crate) struct GamedataParsedScript {
  pub(crate) path: String,
  /// The parse every check reading the script shares, the script or the diagnostics LuaJIT refuses it with, or why it
  /// did not read.
  pub(crate) parse: XrfResult<Arc<XrfResult<XRayLuaScript>>>,
}

impl GamedataParsedScript {
  /// The script, or `None` where it did not read or does not parse.
  pub(crate) fn get_script(&self) -> Option<&XRayLuaScript> {
    self.parse.as_ref().ok()?.as_ref().as_ref().ok()
  }
}
