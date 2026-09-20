use serde::Serialize;
use xrf_vfs::XrayResolution;

/// One texture a visual's submesh declares, and what the reference came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualTextureDependency {
  pub submesh_index: u32,
  pub reference: String,
  pub resolution: XrayResolution,
}
