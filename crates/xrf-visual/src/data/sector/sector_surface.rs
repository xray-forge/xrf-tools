use serde::Serialize;

/// How one part of a sector is dressed, as the level's shader table names it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorSurface {
  /// Entry of the level's shader table this is dressed by.
  pub shader_id: u16,
  /// The engine shader that entry names, absent when the level carries no table.
  pub shader_name: Option<String>,
  /// The base texture that entry names, absent for the same reason.
  pub texture_name: Option<String>,
  /// The one the deferred renderer binds as `s_hemi`, out of which it reads hemisphere and sun occlusion.
  pub hemi: Option<String>,
}
