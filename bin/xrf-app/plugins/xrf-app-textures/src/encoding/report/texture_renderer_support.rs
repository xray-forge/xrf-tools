use serde::Serialize;

use crate::encoding::TextureFormatSupport;

/// One renderer's answer about a format.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureRendererSupport {
  pub renderer: String,
  pub support: TextureFormatSupport,
}
