//! Whether a `[header]` the engine is handed can be loaded at all.
//!
//! Every rule here is a crash, read out of `xray-16/src/xrCore/LocatorAPI.cpp` and `Xr_ini.cpp`, not a matter of
//! taste. `CInifile::r_string` ends in `xrDebug::Fatal` when a key is missing, and `CLocatorAPI::LoadArchive` asks
//! for `entry_point` unconditionally while `ProcessArchive` asks for `auto_load` before it. A volume missing either
//! does not mount somewhere unexpected — it takes the game down on load.

use xrf_error::{XrfError, XrfResult};

use crate::pack::config::ArchivePackHeaderEntry;

/// The section every reader looks for first.
const HEADER_SECTION: &str = "[header]";

/// Keys `CLocatorAPI` reads without checking whether they are there.
const REQUIRED_KEYS: [&str; 2] = ["auto_load", "entry_point"];

/// Refuse a header the engine would fail to load.
///
/// # Errors
///
/// Returns an invalid error for a missing `[header]` section, a missing `auto_load` or `entry_point`, and an
/// `entry_point` that is neither the bare `gamedata` literal nor an alias reference beginning with `$`.
pub(crate) fn require_loadable_header(header: &str) -> XrfResult<()> {
  if !header
    .lines()
    .any(|line| line.trim().eq_ignore_ascii_case(HEADER_SECTION))
  {
    return Err(XrfError::new_invalid_error(format!(
      "The archive header names no '{HEADER_SECTION}' section, and every reader of it asks for that section by name \
       before reading a key."
    )));
  }

  let entries: Vec<ArchivePackHeaderEntry> = ArchivePackHeaderEntry::split(header);

  for key in REQUIRED_KEYS {
    if !entries.iter().any(|entry| entry.key.eq_ignore_ascii_case(key)) {
      return Err(XrfError::new_invalid_error(format!(
        "The archive header names no '{key}'. 'CLocatorAPI' reads it without checking whether it is there, so a \
         volume without it stops the engine on load rather than mounting oddly. Add it, or drop the header entirely \
         to write a volume with no header chunk."
      )));
    }
  }

  let Some(entry_point) = entries
    .iter()
    .find(|entry| entry.key.eq_ignore_ascii_case("entry_point"))
  else {
    return Ok(());
  };

  // `LoadArchive` special-cases the bare literal and otherwise asserts the value opens with an alias.
  if !entry_point.value.eq_ignore_ascii_case("gamedata") && !entry_point.value.starts_with('$') {
    return Err(XrfError::new_invalid_error(format!(
      "Archive header entry point '{}' is neither 'gamedata' nor an alias such as '$fs_root$\\gamedata\\'. \
       'CLocatorAPI::LoadArchive' asserts on the leading '$', so the engine stops on load.",
      entry_point.value
    )));
  }

  Ok(())
}
