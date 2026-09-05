use serde::Serialize;

use crate::data::xray_detail_usage::XrayDetailUsage;

/// The detail texture a descriptor names, and whether the engine applies it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayMaterialDetail {
  /// Detail texture path without extension, engine-style, verbatim from the chunk.
  pub name: String,
  pub scale: f32,
  /// `None` when the name is authored but neither detail flag is set, which the engine treats as no association
  /// (`TextureDescrManager.cpp`). Reported rather than dropped, because dead authoring is a thing to fix.
  pub usage: Option<XrayDetailUsage>,
}
