use std::fmt::Debug;
use std::path::Path;

use serde::Serialize;
use xrf_error::XrfResult;

use crate::{XrayAssetContainer, XrayPathCollision};

/// The storage kind backing a mount.
///
/// It distinguishes loose filesystem entries from entries inside archive volumes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XraySourceKind {
  /// Loose files under a directory.
  Directory,
  /// Entries inside a set of `.db` archive volumes.
  Archive,
}

/// Names a source after the directory or volume set it was opened from, for [`XrayAssetSource::label`].
///
/// The crate's answer for both of its own sources, so a diagnostic reads `textures` rather than a whole install path.
/// Falls back to the path itself for a root with no final component, the one case `file_name` declines.
pub(crate) fn label_from_path(path: &Path) -> String {
  path
    .file_name()
    .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().to_string())
}

/// An asset source addressed by normalized paths relative to itself.
///
/// [`crate::XrayMount`] strips its logical base before calling the source, allowing the same source type to back a root or
/// a subtree. Sources are `Send + Sync` because the mounted VFS is shared across application commands.
///
/// This is the extension seam: a source for a format this crate does not own belongs beside that format and implements
/// this trait, then mounts with [`crate::XrayVfs::mount`]. Key entries with [`crate::XrayLogicalPath::normalize`] and
/// scope enumeration with [`crate::XrayLogicalPath::is_component_prefix`], so lookups behave the same whichever kind of
/// source answers; [`XrayArchiveSource`](crate::XrayArchiveSource) is the in-crate exemplar.
pub trait XrayAssetSource: Debug + Send + Sync {
  /// Short name for reporting, such as a directory or volume-set name.
  fn get_label(&self) -> &str;

  /// Classifies the source's physical storage.
  fn get_kind(&self) -> XraySourceKind;

  /// Whether existing entries can be overwritten through [`Self::write`].
  fn is_writable(&self) -> bool;

  /// Returns the directory root or the directory containing the archive volumes.
  fn get_root_path(&self) -> &Path;

  /// Checks whether the source contains a source-relative logical path.
  ///
  /// Must answer exactly when [`Self::locate`] does. [`crate::XrayVfs`] picks a winning mount with this and then reads,
  /// sizes or locates through that same mount, so a source answering the two differently would send a read to a mount
  /// the preceding lookup did not choose. The default derives it from `locate` so they cannot drift; override it when
  /// membership can be answered without building a container, as both of this crate's sources do.
  fn contains(&self, path: &str) -> bool {
    self.locate(path).is_some()
  }

  /// Locates an entry in its physical container.
  ///
  /// Returns `None` when the source does not contain `path`.
  fn locate(&self, path: &str) -> Option<XrayAssetContainer>;

  /// Reads an existing entry.
  fn read(&self, path: &str) -> XrfResult<Vec<u8>>;

  /// Overwrites an existing entry when [`Self::is_writable`] is true.
  fn write(&self, path: &str, bytes: &[u8]) -> XrfResult<()>;

  /// Creates an entry the source does not currently expose, when writable.
  ///
  /// Implementations may leave mount-time indexes stale. [`crate::XrayVfs::write_override`] remounts after creation.
  fn create(&self, path: &str, bytes: &[u8]) -> XrfResult<()>;

  /// Enumerates source-relative logical paths, optionally restricted to a component prefix.
  fn list_entries<'a>(&'a self, prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a>;

  /// Size in bytes of an entry this source holds, without reading it.
  ///
  /// Answered from metadata rather than from a read, because the callers are size gates: a level's cform is checked against
  /// its header size precisely to avoid parsing a truncated file. An archive knows this from its name table, so neither
  /// container has to decompress anything.
  fn get_size(&self, path: &str) -> Option<u64>;

  /// Files this source holds but cannot reach, because another file already claims their engine identity.
  ///
  /// Defaults to none, which is correct for a source whose names are unique by construction: an archive volume keys entries
  /// by name, so two entries cannot collide inside one set.
  fn get_collisions(&self) -> &[XrayPathCollision] {
    &[]
  }
}
