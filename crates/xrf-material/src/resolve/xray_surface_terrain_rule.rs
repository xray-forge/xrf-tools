use xrf_shaders::{ShaderBlender, ShaderBlenderClass};

use crate::data::xray_surface_terrain::XraySurfaceTerrain;
use crate::data::xray_surface_terrain_layer::XraySurfaceTerrainLayer;

/// Which blender classes the deferred renderers draw as terrain, and what they lay over its base.
pub(crate) struct XraySurfaceTerrainRule;

impl XraySurfaceTerrainRule {
  /// The marker `CBlender_BmmD::Save` writes its detail group under, the four details after its own.
  const MARKER: &'static str = "Detail map";

  /// The properties naming the details the mask's red, green, blue and alpha weigh.
  const LAYERS: [&'static str; 4] = ["R2-R", "R2-G", "R2-B", "R2-A"];

  /// The terrain a class lays over its base, or `None` for a class the deferred renderers do not compile as
  /// `CBlender_BmmD` (`r2_blenders.cpp`: `B_LmBmmD` and `B_BmmD`), or a blender naming no details, written before
  /// its version 3.
  pub(crate) fn describe(blender: &ShaderBlender, base: Option<&str>) -> Option<XraySurfaceTerrain> {
    if !matches!(
      blender.class,
      ShaderBlenderClass::LM_BMM_D | ShaderBlenderClass::BMM_D_OLD
    ) {
      return None;
    }

    let base: &str = base.filter(|it| !it.is_empty())?;
    let mut layers = Self::LAYERS.iter().map(|property| {
      blender
        .texture(Self::MARKER, property)
        .filter(|reference| !reference.is_empty() && *reference != ShaderBlender::NULL_TEXTURE)
        .map(|reference| XraySurfaceTerrainLayer {
          reference: reference.to_owned(),
          bump: format!("{reference}_bump"),
        })
    });
    let layers: [XraySurfaceTerrainLayer; 4] = [layers.next()??, layers.next()??, layers.next()??, layers.next()??];

    Some(XraySurfaceTerrain {
      mask: format!("{base}_mask"),
      layers,
    })
  }
}
