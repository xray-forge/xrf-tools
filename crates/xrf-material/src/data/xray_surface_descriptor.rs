use serde::Serialize;
use xrf_vfs::XrayAsset;

use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_detail::XraySurfaceDetail;
use crate::data::xray_surface_draw::XraySurfaceDraw;

/// How the renderer draws one surface, resolved from the shader name it declares and the textures it dresses with.
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
  /// The detail texture modulating its diffuse, `None` for a class the engine never details or a base texture whose
  /// descriptor associates none.
  pub detail: Option<XraySurfaceDetail>,
}

impl XraySurfaceDescriptor {
  /// A surface drawn opaque and undetailed, for every reason short of a blender that asks for either.
  pub fn opaque(library: Option<XrayAsset>, declaration: XraySurfaceDeclaration) -> Self {
    Self {
      library,
      declaration,
      draw: XraySurfaceDraw::Opaque,
      detail: None,
    }
  }
}
