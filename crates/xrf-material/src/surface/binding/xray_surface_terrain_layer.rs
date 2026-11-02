use serde::Serialize;

/// One of the four details a terrain lays over its base where its mask's channel says, with the bump it is lit by.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceTerrainLayer {
  /// Detail texture reference, engine-style, without extension.
  pub reference: String,
  /// Its bump, the reference with `_bump` after it.
  pub bump: String,
}
