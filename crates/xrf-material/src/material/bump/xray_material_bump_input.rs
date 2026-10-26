use serde::Serialize;
use xrf_vfs::XrayResolution;

/// One of the two textures a bump declaration makes the renderer bind, and what binding it came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayMaterialBumpInput {
  /// The engine path the renderer asks for, verbatim.
  pub reference: String,
  pub resolution: XrayResolution,
}

impl XrayMaterialBumpInput {
  /// Whether the file on the surface is the one the declaration named.
  pub fn is_declared_file(&self) -> bool {
    matches!(self.resolution, XrayResolution::Resolved { .. })
  }
}
