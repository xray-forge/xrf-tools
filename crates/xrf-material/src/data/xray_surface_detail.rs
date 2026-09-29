use serde::Serialize;

use crate::data::xray_material_bump::XrayMaterialBump;

/// The detail texture a surface modulates its diffuse with, and how densely it is laid over it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceDetail {
  /// Detail texture reference, engine-style, without extension.
  pub reference: String,
  /// Times it repeats across the surface's base coordinate, `dt_params.xyz` (`TextureDescrManager.cpp`).
  pub scale: f32,
  /// The bump pair the detail texture's own descriptor names, added to the surface's where its usage bumps and the
  /// surface binds a pair of its own (`uber_deffer.cpp`, `_db`); `None` otherwise.
  pub bump: Option<XrayMaterialBump>,
}
