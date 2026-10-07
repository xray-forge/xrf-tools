use std::sync::Arc;

use glam::{UVec4, Vec3};
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_visual::SectorSurface;

use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_terrain_slots::StaticTerrainSlots;
use crate::scene::static_scene::static_water_surface::build_water_surface;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// The texture descriptor's default lighting model: Blinn, at full weight.
const DEFAULT_MATERIAL: f32 = 1.0;

/// The environment-mapped model class's tag.
const ENVIRONMENT_MAPPED_CLASS: &str = "MODELEbB";

/// Turns of the golden angle between consecutive shader ids, which spreads their colours rather than grouping them.
const HUE_STEP: f32 = 137.508;

/// A shader table entry as the static draws wear it, asking for its textures; `None` for an invisible one, which no
/// static draw draws. A composited wall mark is laid into the G-buffer's albedo rather than over the lit frame.
pub fn build_static_surface(
  surface: &SectorSurface,
  descriptor: Option<&XraySurfaceDescriptor>,
  textures: &mut TextureCache,
  source: &Arc<dyn RenderAssetSource>,
) -> Option<(StaticSurface, StaticClass)> {
  let (class, reference, blend): (StaticClass, u8, u32) = match descriptor.map(|it| it.draw) {
    None | Some(XraySurfaceDraw::Opaque) => (StaticClass::Opaque, XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE, 0),
    Some(XraySurfaceDraw::AlphaTested { reference }) => (StaticClass::CutOut, reference, 0),
    Some(XraySurfaceDraw::Blended { reference }) => (StaticClass::Composited, reference, 0),
    Some(XraySurfaceDraw::Added { reference, is_weighted }) => (
      StaticClass::Composited,
      reference,
      StaticSurface::IS_ADDED | if is_weighted { StaticSurface::IS_WEIGHTED } else { 0 },
    ),
    Some(XraySurfaceDraw::Multiplied { is_doubled }) => (
      StaticClass::Composited,
      0,
      StaticSurface::IS_MULTIPLIED | if is_doubled { StaticSurface::IS_DOUBLED } else { 0 },
    ),
    Some(XraySurfaceDraw::Water { is_soft }) => {
      let descriptor: &XraySurfaceDescriptor = descriptor?;
      let color: Vec3 = to_surface_color(surface.shader_id);

      return Some((
        build_water_surface(surface, descriptor, is_soft, textures, source, color),
        StaticClass::Water,
      ));
    }
    Some(_) => return None,
  };

  let class: StaticClass = if class == StaticClass::Composited && descriptor.is_some_and(is_wallmark) {
    StaticClass::Wallmark
  } else {
    class
  };

  let mut request = |reference: Option<&str>, role: TextureRole| -> Option<u32> {
    reference
      .filter(|it| !it.is_empty())
      .map(|it| textures.request(it, role, source))
  };
  let detail = descriptor
    .and_then(|it| it.detail.as_ref())
    .filter(|detail| detail.scale.is_finite());
  // A terrain lays its own four details and bumps, at its detail's tiling; its single detail and bump pair are the
  // low-quality path's, which it does not draw.
  let is_terrain: bool = descriptor.is_some_and(|it| it.terrain.is_some());
  let bump = descriptor.and_then(|it| it.bump.as_ref()).filter(|_| !is_terrain);
  let detail_bump = detail.and_then(|detail| detail.bump.as_ref()).filter(|_| !is_terrain);
  let slots: [Option<u32>; 7] = [
    request(surface.texture_name.as_deref(), TextureRole::Base),
    request(
      detail.filter(|_| !is_terrain).map(|it| it.reference.as_str()),
      TextureRole::Detail,
    ),
    request(bump.map(|it| it.bump.reference.as_str()), TextureRole::Bump),
    request(
      bump.map(|it| it.companion.reference.as_str()),
      TextureRole::BumpCompanion,
    ),
    request(detail_bump.map(|it| it.bump.reference.as_str()), TextureRole::Bump),
    request(
      detail_bump.map(|it| it.companion.reference.as_str()),
      TextureRole::BumpCompanion,
    ),
    request(surface.hemi.as_deref(), TextureRole::Hemi),
  ];
  let mut flags: u32 = 0;

  for (slot, flag) in [
    (StaticSurface::BASE, StaticSurface::HAS_BASE),
    (StaticSurface::DETAIL, StaticSurface::HAS_DETAIL),
    (StaticSurface::BUMP, StaticSurface::HAS_BUMP),
    (StaticSurface::DETAIL_BUMP, StaticSurface::HAS_DETAIL_BUMP),
    (StaticSurface::HEMI, StaticSurface::HAS_HEMI),
  ] {
    if slots[slot].is_some() {
      flags |= flag;
    }
  }

  // A bump binds as a pair, and a detail's bump adds only to the surface's own.
  if slots[StaticSurface::BUMP_COMPANION].is_none() {
    flags &= !(StaticSurface::HAS_BUMP | StaticSurface::HAS_DETAIL_BUMP);
  }

  if slots[StaticSurface::DETAIL_BUMP_COMPANION].is_none() {
    flags &= !StaticSurface::HAS_DETAIL_BUMP;
  }

  // A composited surface tests its own reference where it has one.
  if class == StaticClass::CutOut || (class == StaticClass::Composited && reference > 0) {
    flags |= StaticSurface::IS_CUT_OUT;
  }

  flags |= blend;

  if descriptor.is_some_and(is_environment_mapped) {
    flags |= StaticSurface::IS_ENVIRONMENT_MAPPED;
  }

  if descriptor.is_some_and(|it| it.is_object_lod) {
    flags |= StaticSurface::IS_STILL;
  }

  if descriptor.is_some_and(|it| it.is_emissive) {
    flags |= StaticSurface::IS_EMISSIVE;
  }

  if descriptor.is_some_and(|it| it.is_shadowless) {
    flags |= StaticSurface::IS_SHADOWLESS;
  }

  let terrain: StaticTerrainSlots = match descriptor.and_then(|it| it.terrain.as_ref()) {
    Some(terrain) if flags & StaticSurface::HAS_BASE != 0 => {
      flags |= StaticSurface::IS_TERRAIN;

      StaticTerrainSlots {
        details: UVec4::from_array(
          terrain
            .layers
            .each_ref()
            .map(|layer| textures.request(&layer.reference, TextureRole::Detail, source)),
        ),
        bumps: UVec4::from_array(
          terrain
            .layers
            .each_ref()
            .map(|layer| textures.request(&layer.bump, TextureRole::Bump, source)),
        ),
        mask: textures.request(&terrain.mask, TextureRole::TerrainMask, source),
        _pad: [0; 3],
      }
    }
    _ => StaticTerrainSlots::default(),
  };
  let environment: u32 = descriptor
    .and_then(|it| it.environment.as_deref())
    .filter(|_| flags & StaticSurface::IS_ENVIRONMENT_MAPPED != 0)
    .map_or(0, |reference| textures.request_environment(reference));

  let mut texture_slots: [u32; 8] = [0; 8];

  for (index, slot) in slots.iter().enumerate() {
    texture_slots[index] = slot.unwrap_or(0);
  }

  texture_slots[StaticSurface::ENVIRONMENT] = environment;

  let material: f32 = descriptor.map_or(DEFAULT_MATERIAL, |it| it.material);

  Some((
    StaticSurface {
      tiling: 1.0,
      detail_scale: detail.map_or(1.0, |it| it.scale),
      alpha_reference: reference as f32 / 255.0,
      slice: (material + 0.5) / 4.0,
      color: to_surface_color(surface.shader_id),
      flags,
      textures: texture_slots,
      terrain,
    },
    class,
  ))
}

/// What `details_lod.s` appends to the atlas for its `s_hemi`.
const IMPOSTOR_COMPANION_SUFFIX: &str = "_nm";

/// The material slice `lod.ps` writes: the default lighting model.
const IMPOSTOR_MATERIAL: f32 = 0.0;

/// A run of impostors' surface, asking for its atlas and the atlas's `_nm` companion.
pub fn build_impostor_surface(
  surface: &SectorSurface,
  textures: &mut TextureCache,
  source: &Arc<dyn RenderAssetSource>,
) -> StaticSurface {
  let mut texture_slots: [u32; 8] = [0; 8];
  let mut flags: u32 = 0;

  if let Some(atlas) = surface.texture_name.as_deref().filter(|it| !it.is_empty()) {
    texture_slots[StaticSurface::BASE] = textures.request(atlas, TextureRole::Base, source);
    texture_slots[StaticSurface::HEMI] = textures.request(
      &format!("{atlas}{IMPOSTOR_COMPANION_SUFFIX}"),
      TextureRole::ImpostorCompanion,
      source,
    );
    flags |= StaticSurface::HAS_BASE | StaticSurface::HAS_HEMI;
  }

  StaticSurface {
    tiling: 1.0,
    detail_scale: 1.0,
    alpha_reference: 0.0,
    slice: (IMPOSTOR_MATERIAL + 0.5) / 4.0,
    color: to_surface_color(surface.shader_id),
    flags,
    textures: texture_slots,
    terrain: StaticTerrainSlots::default(),
  }
}

fn is_wallmark(descriptor: &XraySurfaceDescriptor) -> bool {
  matches!(
    descriptor.declaration,
    XraySurfaceDeclaration::Scripted { is_wallmark: true, .. }
  )
}

/// `B_MODEL_EbB`'s, which blends toward an environment map where its base is thin.
fn is_environment_mapped(descriptor: &XraySurfaceDescriptor) -> bool {
  matches!(&descriptor.declaration, XraySurfaceDeclaration::Described { class, .. } if class == ENVIRONMENT_MAPPED_CLASS)
}

/// A stable colour per shader table entry, from a hue its id decides, so a surface reads the same in every sector.
fn to_surface_color(shader_id: u16) -> Vec3 {
  let hue: f32 = (shader_id as f32 * HUE_STEP) % 360.0;
  let (saturation, lightness): (f32, f32) = (0.45, 0.6);
  let reach: f32 = saturation * lightness.min(1.0 - lightness);
  let channel = |offset: f32| -> f32 {
    let turn: f32 = (offset + hue / 30.0) % 12.0;

    lightness - reach * (turn - 3.0).min(9.0 - turn).clamp(-1.0, 1.0)
  };

  Vec3::new(channel(0.0), channel(8.0), channel(4.0))
}
