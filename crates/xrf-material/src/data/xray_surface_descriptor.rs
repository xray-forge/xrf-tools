use serde::Serialize;
use xrf_vfs::XrayAsset;

use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_draw::XraySurfaceDraw;

/// How the renderer draws one surface, resolved from the shader name it declares.
///
/// The counterpart of [`crate::XrayMaterialDescriptor`], which answers the same question for a texture from its `.thm`.
/// Between them they are what a surface is made of: the shader decides whether alpha is read and how, the descriptor
/// decides what is bound beside the diffuse.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceDescriptor {
  /// The `shaders.xr` the answer was read from, or `None` when no root holds one.
  pub library: Option<XrayAsset>,
  pub declaration: XraySurfaceDeclaration,
  /// What to draw. Always answerable: a surface nothing could be read for is drawn the way the engine draws one whose
  /// shader it could not resolve, which is opaque.
  pub draw: XraySurfaceDraw,
}

impl XraySurfaceDescriptor {
  /// A surface drawn opaque, for every reason short of a blender that asks for it.
  pub fn opaque(library: Option<XrayAsset>, declaration: XraySurfaceDeclaration) -> Self {
    Self {
      library,
      declaration,
      draw: XraySurfaceDraw::Opaque,
    }
  }

  /// Whether the surface reads its texture's alpha channel.
  pub fn is_alpha_read(&self) -> bool {
    self.draw.is_alpha_read()
  }
}
