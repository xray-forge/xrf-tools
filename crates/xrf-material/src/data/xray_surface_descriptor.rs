use serde::Serialize;
use xrf_vfs::XrayAsset;

use crate::data::xray_material_bump::XrayMaterialBump;
use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_detail::XraySurfaceDetail;
use crate::data::xray_surface_draw::XraySurfaceDraw;
use crate::data::xray_surface_sampler::XraySurfaceSampler;
use crate::data::xray_surface_terrain::XraySurfaceTerrain;

/// How the renderer draws one surface, resolved from the shader name it declares and the textures it dresses with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceDescriptor {
  /// The shader the surface named, as its level or mesh spells it, or `None` where it named none.
  pub shader: Option<String>,
  /// The textures the entry dresses with, in the order it names them: the base, then whatever its class binds.
  pub textures: Vec<String>,
  /// The `shaders.xr` the answer was read from, or `None` when no root holds one.
  pub library: Option<XrayAsset>,
  pub declaration: XraySurfaceDeclaration,
  /// What to draw. Always answerable: a surface nothing could be read for is drawn the way the engine draws one whose
  /// shader it could not resolve, which is opaque.
  pub draw: XraySurfaceDraw,
  /// The detail texture modulating its diffuse, `None` for a class the engine never details or a base texture whose
  /// descriptor associates none.
  pub detail: Option<XraySurfaceDetail>,
  /// The four details and mask a terrain class lays over its base instead, `None` for any other class.
  pub terrain: Option<XraySurfaceTerrain>,
  /// The texture files a scripted surface binds by sampler, in every element the renderer compiles for it; none for a
  /// surface the blender library describes, whose class binds by slot.
  pub samplers: Vec<XraySurfaceSampler>,
  /// The bump pair its base texture's descriptor declares, for a class that binds one.
  pub bump: Option<XrayMaterialBump>,
  /// The lighting model its base texture's descriptor sets, `m_material`.
  pub material: f32,
  /// The cube an environment-mapped class mixes its base toward where its alpha is thin, `oT2_Name`; `None` for a
  /// class binding none.
  pub environment: Option<String>,
  /// Whether its base is sampled clamped to the edge rather than wrapped, `Texture clamp`.
  pub is_texture_clamped: bool,
  /// Whether it also draws into the distortion target, by a script's `l_special` pass.
  pub is_distorting: bool,
  /// `B_TREE`'s `Object LOD`: an object the level stores as a tree, drawn by `tree_s`, which the wind leaves standing.
  pub is_object_lod: bool,
}

impl XraySurfaceDescriptor {
  /// A surface drawn opaque and undetailed, for every reason short of a blender that asks for either.
  pub fn opaque(library: Option<XrayAsset>, declaration: XraySurfaceDeclaration) -> Self {
    Self {
      shader: None,
      textures: Vec::new(),
      library,
      declaration,
      draw: XraySurfaceDraw::Opaque,
      detail: None,
      terrain: None,
      samplers: Vec::new(),
      bump: None,
      material: XrayMaterialDescriptor::DEFAULT_MATERIAL,
      environment: None,
      is_texture_clamped: false,
      is_distorting: false,
      is_object_lod: false,
    }
  }

  /// The texture its script binds to a sampler of an element, none where it binds none there.
  pub fn find_sampler(&self, element: &str, name: &str) -> Option<&str> {
    self
      .samplers
      .iter()
      .find(|it| it.element == element && it.name == name)
      .map(|it| it.texture.as_str())
      .filter(|it| !it.is_empty())
  }

  /// The same surface, saying which table entry it answered for.
  pub fn resolved_from(mut self, shader: &str, textures: &[String]) -> Self {
    self.shader = Some(shader.to_owned());
    self.textures = textures.to_vec();

    self
  }
}
