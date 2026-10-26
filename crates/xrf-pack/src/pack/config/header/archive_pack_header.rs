//! Whether a `[header]` the engine is handed can be loaded at all.

use xrf_error::{XrfError, XrfResult};

use crate::pack::config::ArchivePackHeaderEntry;

/// The `[header]` text chunk 666 carries: the one every volume gets by default, and what makes any of them loadable.
pub(crate) struct ArchivePackHeader;

impl ArchivePackHeader {
  /// The section line every reader looks for first.
  pub(crate) const SECTION: &str = "[header]";

  /// Where a packed `gamedata` tree mounts, which is what nearly every archive is.
  pub(crate) const DEFAULT_ENTRY_POINT: &str = "$fs_root$\\gamedata\\";

  /// Keys `CLocatorAPI` reads without checking whether they are there.
  const REQUIRED_KEYS: [&str; 2] = ["auto_load", "entry_point"];

  /// The header written unless a configuration supplies its own.
  pub(crate) fn default_text() -> String {
    format!(
      "{}\r\nauto_load = true\r\nentry_point = {}\r\n",
      Self::SECTION,
      Self::DEFAULT_ENTRY_POINT
    )
  }

  /// Refuse a header the engine would fail to load.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a missing `[header]` section, a missing `auto_load` or `entry_point`, and an
  /// `entry_point` that is neither the bare `gamedata` literal nor an alias reference beginning with `$`.
  pub(crate) fn require_loadable(header: &str) -> XrfResult {
    if !header
      .lines()
      .any(|line| line.trim().eq_ignore_ascii_case(Self::SECTION))
    {
      return Err(XrfError::new_invalid_error(format!(
        "The archive header names no '{}' section, and every reader of it asks for that section by name before \
         reading a key.",
        Self::SECTION
      )));
    }

    let entries: Vec<ArchivePackHeaderEntry> = ArchivePackHeaderEntry::split(header);

    for key in Self::REQUIRED_KEYS {
      if !entries.iter().any(|entry| entry.key.eq_ignore_ascii_case(key)) {
        return Err(XrfError::new_invalid_error(format!(
          "The archive header names no '{key}'. 'CLocatorAPI' reads it without checking whether it is there, so a \
           volume without it stops the engine on load rather than mounting oddly. Add it, or drop the header \
           entirely to write a volume with no header chunk."
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
}
