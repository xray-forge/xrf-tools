use serde::Serialize;

/// A descriptor field that names another texture.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TextureReferenceField {
  /// `bump_name`, the bump pair the engine binds.
  Bump,
  /// `detail_name`, the texture tiled over this one up close.
  Detail,
}
