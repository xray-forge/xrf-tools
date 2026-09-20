use serde::Serialize;

/// The detail texture a surface modulates its diffuse with, and how densely it is laid over it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceDetail {
  /// Detail texture reference, engine-style, without extension.
  pub reference: String,
  /// Times it repeats across the surface's base coordinate, `dt_params.xyz` (`TextureDescrManager.cpp`).
  pub scale: f32,
}
