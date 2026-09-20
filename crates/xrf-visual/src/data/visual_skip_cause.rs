use serde::Serialize;

/// Why a submesh produced no geometry, graded so a caller does not read the message to find out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VisualSkipCause {
  /// Geometry is stored in a form the packer does not handle, such as a shared vertex or index
  /// container living outside the file.
  Unsupported,
  /// Geometry contradicts itself, such as a detail level reaching past the index buffer it indexes.
  Malformed,
}
