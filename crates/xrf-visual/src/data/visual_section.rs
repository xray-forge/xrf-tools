use serde::Serialize;

/// Byte range of one packed attribute inside a visual's geometry buffer.
///
/// Both values are byte counts rather than element counts, so a consumer builds a typed array view
/// directly from them. The packer aligns every offset to four bytes for `Float32Array` and
/// `Uint16Array` views.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSection {
  pub byte_offset: u32,
  pub byte_length: u32,
}

/// The slice of an index buffer that draws one detail level.
///
/// Element offsets into the index buffer, not bytes, because that is what a draw call takes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualDrawRange {
  pub start: u32,
  pub count: u32,
}
