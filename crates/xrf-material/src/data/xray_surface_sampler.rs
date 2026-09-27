use serde::Serialize;

/// A texture file a surface's script binds to one of its samplers.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceSampler {
  /// The script function whose pass binds it: `normal` for the surface itself, `l_special` for its distortion.
  pub element: String,
  /// The sampler's name in the shader, `s_nmap` and its kin.
  pub name: String,
  /// Texture reference, engine-style, without extension.
  pub texture: String,
}
