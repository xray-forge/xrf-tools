use serde::Serialize;

/// Which side of a comparison an entry was read from, and how big it was there.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchSide {
  /// Position in the report's `origins` of the volume set or loose root this was read from.
  ///
  /// An index rather than the path itself: a comparison names a handful of origins over tens of thousands of entries,
  /// so spelling one out per side is the bulk of a large report and says nothing a shared table cannot.
  pub origin: u32,
  /// Unpacked payload size, from the name table for an archived entry and from metadata for a loose one.
  pub size: u64,
}
