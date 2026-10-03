use std::sync::Arc;

use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_visual::SectorSurface;

use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_water_surface::build_water_surface;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// The texture descriptor's default lighting model: Blinn, at full weight.
const DEFAULT_MATERIAL: f32 = 1.0;

/// Turns of the golden angle between consecutive shader ids, which spreads their colours rather than grouping them.
const HUE_STEP: f32 = 137.508;

/// A shader table entry as the static draws wear it, asking for its textures; `None` for one no static draw draws:
/// composited, invisible or a wall mark, drawn by passes of their own.
pub fn build_static_surface(
  surface: &SectorSurface,
  descriptor: Option<&XraySurfaceDescriptor>,
  textures: &mut TextureCache,
  source: &Arc<dyn RenderAssetSource>,
) -> Option<(StaticSurface, StaticClass)> {
  let (class, reference): (StaticClass, u8) = match descriptor.map(|it| it.draw) {
    None | Some(XraySurfaceDraw::Opaque) => (StaticClass::Opaque, XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE),
    Some(XraySurfaceDraw::AlphaTested { reference }) => (StaticClass::CutOut, reference),
    Some(XraySurfaceDraw::Water { is_soft }) => {
      let descriptor: &XraySurfaceDescriptor = descriptor?;
      let color: [f32; 3] = to_surface_color(surface.shader_id);

      return Some((
        build_water_surface(surface, descriptor, is_soft, textures, source, color),
        StaticClass::Water,
      ));
    }
    Some(_) => return None,
  };

  if descriptor.is_some_and(is_wallmark) {
    return None;
  }

  let mut request = |reference: Option<&str>, role: TextureRole| -> Option<u32> {
    reference
      .filter(|it| !it.is_empty())
      .map(|it| textures.request(it, role, source))
  };
  let detail = descriptor
    .and_then(|it| it.detail.as_ref())
    .filter(|detail| detail.scale.is_finite());
  let bump = descriptor.and_then(|it| it.bump.as_ref());
  let detail_bump = detail.and_then(|detail| detail.bump.as_ref());
  let slots: [Option<u32>; 7] = [
    request(surface.texture_name.as_deref(), TextureRole::Base),
    request(detail.map(|it| it.reference.as_str()), TextureRole::Detail),
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

  if class == StaticClass::CutOut {
    flags |= StaticSurface::IS_CUT_OUT;
  }

  let mut texture_slots: [u32; 8] = [0; 8];

  for (index, slot) in slots.iter().enumerate() {
    texture_slots[index] = slot.unwrap_or(0);
  }

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
  }
}

fn is_wallmark(descriptor: &XraySurfaceDescriptor) -> bool {
  matches!(
    descriptor.declaration,
    XraySurfaceDeclaration::Scripted { is_wallmark: true, .. }
  )
}

/// A stable colour per shader table entry, from a hue its id decides, so a surface reads the same in every sector.
fn to_surface_color(shader_id: u16) -> [f32; 3] {
  let hue: f32 = (shader_id as f32 * HUE_STEP) % 360.0;
  let (saturation, lightness): (f32, f32) = (0.45, 0.6);
  let reach: f32 = saturation * lightness.min(1.0 - lightness);
  let channel = |offset: f32| -> f32 {
    let turn: f32 = (offset + hue / 30.0) % 12.0;

    lightness - reach * (turn - 3.0).min(9.0 - turn).clamp(-1.0, 1.0)
  };

  [channel(0.0), channel(8.0), channel(4.0)]
}
