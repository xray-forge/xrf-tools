use xrf_error::XrfResult;
use xrf_lua::XRayLuaScript;

/// One runtime script as the checks reading its contents meet it: its logical path, and the script, or why it did not
/// read or parse, which the scripts check reports.
pub(crate) struct GamedataParsedScript {
  pub(crate) path: String,
  pub(crate) script: XrfResult<XRayLuaScript>,
}
