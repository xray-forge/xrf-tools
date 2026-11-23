use serde::{Deserialize, Serialize};
use xrf_archive::{ArchiveHeaderItem, ArchiveMetadata};
use xrf_error::{XrfError, XrfResult};

use crate::pack::config::ArchivePackHeaderEntry;

/// The `[header]` chunk 666 carries into every volume, held as the entries it is written from.
///
/// Every surface chooses a header as entries and the text exists only where a volume is laid out ([`Self::to_text`]),
/// so the format has one writer and [`Self::require_loadable`] judges exactly what the engine will be handed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArchivePackHeader(Vec<ArchivePackHeaderEntry>);

impl ArchivePackHeader {
  /// The section line every reader looks for first.
  const SECTION: &str = "[header]";

  /// Where a packed `gamedata` tree mounts, which is what nearly every archive is.
  const DEFAULT_ENTRY_POINT: &str = "$fs_root$\\gamedata\\";

  const KEY_AUTO_LOAD: &str = "auto_load";

  const KEY_ENTRY_POINT: &str = "entry_point";

  /// Keys `CLocatorAPI` reads without checking whether they are there.
  const REQUIRED_KEYS: [&str; 2] = [Self::KEY_AUTO_LOAD, Self::KEY_ENTRY_POINT];

  pub fn new(entries: Vec<ArchivePackHeaderEntry>) -> Self {
    Self(entries)
  }

  /// The entries in the order they are written.
  pub fn entries(&self) -> &[ArchivePackHeaderEntry] {
    &self.0
  }

  /// This header with `overrides` applied by key, keeping the order its own entries were written in.
  ///
  /// Keys match without case, so `Entry_Point` sets the existing `entry_point` in the spelling the engine finds: it
  /// matches keys exactly, and an override added beside the original would be ignored.
  #[must_use]
  pub fn with_overrides(mut self, overrides: &[ArchivePackHeaderEntry]) -> Self {
    for override_entry in overrides {
      match self
        .0
        .iter_mut()
        .find(|entry| entry.key.eq_ignore_ascii_case(&override_entry.key))
      {
        Some(existing) => existing.value.clone_from(&override_entry.value),
        None => self.0.push(override_entry.clone()),
      }
    }

    self
  }

  /// The text chunk 666 stores: the section line, then one `<key> = <value>` line per entry, each ending in CRLF.
  pub fn to_text(&self) -> String {
    let mut text: String = format!("{}\r\n", Self::SECTION);

    for entry in &self.0 {
      text.push_str(&entry.key);
      text.push_str(" = ");
      text.push_str(&entry.value);
      text.push_str("\r\n");
    }

    text
  }

  /// Refuse a header the engine would fail to load, or would read as something other than its entries.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for an entry the engine reads back differently ([`Self::require_read_as_written`]), a
  /// key named twice, a missing `auto_load` or `entry_point`, and an `entry_point` the engine cannot mount
  /// ([`ArchiveMetadata::is_mountable_entry_point`]).
  pub(crate) fn require_loadable(&self) -> XrfResult {
    for entry in &self.0 {
      Self::require_read_as_written(entry)?;
    }

    self.require_distinct_keys()?;

    // Spelled exactly: `CInifile` lower-cases section names but finds keys with `strcmp`.
    for key in Self::REQUIRED_KEYS {
      if !self.0.iter().any(|entry| entry.key == key) {
        return Err(XrfError::new_invalid_error(format!(
          "The archive header names no '{key}'. 'CLocatorAPI' reads it without checking whether it is there, so a \
           volume without it stops the engine on load rather than mounting oddly. Add it, or drop the header \
           entirely to write a volume with no header chunk."
        )));
      }
    }

    let Some(entry_point) = self.0.iter().find(|entry| entry.key == Self::KEY_ENTRY_POINT) else {
      return Ok(());
    };

    if !ArchiveMetadata::is_mountable_entry_point(&entry_point.value) {
      return Err(XrfError::new_invalid_error(format!(
        "Archive header entry point '{}' is neither 'gamedata' nor an alias followed by a path, such as \
         '$fs_root$\\gamedata\\'. 'CLocatorAPI::LoadArchive' asserts on the leading '$' and reads the path after the \
         alias's first '\\', so the engine stops on load or reads past the value.",
        entry_point.value
      )));
    }

    Ok(())
  }

  /// Refuse an entry whose own line the engine reads as anything but that key and value.
  ///
  /// Judged by reading the line back as `CInifile` does rather than by listing forbidden characters, so a line break, a
  /// comment opened outside quotes, an `=` in the key, a key opening a section, and whitespace the engine drops from an
  /// unquoted value are all caught by one comparison. A value opening a quote it never closes is refused on its own:
  /// the engine reads on into the following lines, which a single line cannot show.
  fn require_read_as_written(entry: &ArchivePackHeaderEntry) -> XrfResult {
    let line: String = format!("{} = {}", entry.key, entry.value);

    if entry.value.matches('"').count() % 2 == 1 {
      return Err(XrfError::new_invalid_error(format!(
        "Archive header entry '{line}' opens a quote it never closes, so the engine reads the lines after it into \
         the same value. Close the quote."
      )));
    }

    let read: Vec<ArchiveHeaderItem> =
      ArchiveMetadata::read_header(&Self(vec![entry.clone()]).to_text()).unwrap_or_default();

    if let [item] = read.as_slice()
      && item.key == entry.key
      && item.value == entry.value
    {
      return Ok(());
    }

    let reading: String = match read.as_slice() {
      [] => String::from("no entry at all"),
      items => items
        .iter()
        .map(|item| format!("'{} = {}'", item.key, item.value))
        .collect::<Vec<String>>()
        .join(" and "),
    };

    Err(XrfError::new_invalid_error(format!(
      "Archive header entry '{line}' would reach the engine as {reading}. A header line keeps whitespace only inside \
       quotes and holds no line break or comment (';' or '//') outside them, so quote a value with spaces, as \
       \"My Mod\"."
    )))
  }

  /// Refuse a key named twice: the engine keeps only the later of two spelled alike, and two differing in case read as
  /// one key to everyone but the engine, which every editor of the header folds.
  fn require_distinct_keys(&self) -> XrfResult {
    for (index, entry) in self.0.iter().enumerate() {
      let Some(earlier) = self.0[..index]
        .iter()
        .find(|earlier| earlier.key.eq_ignore_ascii_case(&entry.key))
      else {
        continue;
      };

      return Err(if earlier.key == entry.key {
        XrfError::new_invalid_error(format!(
          "The archive header names '{}' twice, and the engine keeps only the later value. Keep one.",
          entry.key
        ))
      } else {
        XrfError::new_invalid_error(format!(
          "The archive header names both '{}' and '{}', which differ only in case. The engine reads them as two keys \
           and finds a required key only as spelled; keep one, in the spelling it means.",
          earlier.key, entry.key
        ))
      });
    }

    Ok(())
  }
}

impl Default for ArchivePackHeader {
  /// The header written unless a configuration supplies its own: loaded at startup, mounted as `gamedata`.
  fn default() -> Self {
    Self(vec![
      ArchivePackHeaderEntry::new(Self::KEY_AUTO_LOAD, "true"),
      ArchivePackHeaderEntry::new(Self::KEY_ENTRY_POINT, Self::DEFAULT_ENTRY_POINT),
    ])
  }
}
