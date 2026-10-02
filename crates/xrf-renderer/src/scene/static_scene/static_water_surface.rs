use std::sync::Arc;

use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor};
use xrf_visual::SectorSurface;

use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// The script functions whose passes bind water's textures: the surface itself, and its distortion.
const BASE_ELEMENT: &str = "normal";
const DISTORTION_ELEMENT: &str = "l_special";

/// Anomaly's water programs (`shaders/r2/water_*.ps` in its gamedata), by the switches each defines before including
/// its `water.ps`: reflecting, specular, transparent, foamed. Any other water program is OpenXRay's.
const ANOMALY_PROGRAMS: [(&str, [bool; 4]); 4] = [
  ("water_regular", [true, true, false, false]),
  ("water_ryaska", [true, true, true, false]),
  ("water_studen", [true, true, false, true]),
  ("water_underground", [false, false, false, false]),
];

/// A water surface as its draws read it: the base, the normal map scrolled twice over, the foam laid in the shallows
/// and the distortion, by the samplers its script binds them to, and its program's model.
pub fn build_water_surface(
  surface: &SectorSurface,
  descriptor: &XraySurfaceDescriptor,
  is_soft: bool,
  textures: &mut TextureCache,
  source: &Arc<dyn RenderAssetSource>,
  color: [f32; 3],
) -> StaticSurface {
  let sampler = |element: &str, name: &str| -> Option<&str> {
    descriptor
      .samplers
      .iter()
      .find(|it| it.element == element && it.name == name)
      .map(|it| it.texture.as_str())
      .filter(|it| !it.is_empty())
  };
  let base: Option<&str> = sampler(BASE_ELEMENT, "s_base").or(surface.texture_name.as_deref());
  let mut texture_slots: [u32; 8] = [0; 8];
  let mut flags: u32 = if is_soft { StaticSurface::IS_SOFT_WATER } else { 0 };

  for (reference, slot, role, flag) in [
    (base, StaticSurface::BASE, TextureRole::Base, StaticSurface::HAS_BASE),
    (
      sampler(BASE_ELEMENT, "s_nmap"),
      StaticSurface::WATER_NORMAL,
      TextureRole::Detail,
      StaticSurface::HAS_WATER_NORMAL,
    ),
    (
      sampler(BASE_ELEMENT, "s_leaves"),
      StaticSurface::FOAM,
      TextureRole::Detail,
      StaticSurface::HAS_FOAM,
    ),
    (
      sampler(DISTORTION_ELEMENT, "s_distort"),
      StaticSurface::DISTORTION,
      TextureRole::Detail,
      StaticSurface::HAS_DISTORTION,
    ),
  ] {
    if let Some(reference) = reference.filter(|it| !it.is_empty()) {
      texture_slots[slot] = textures.request(reference, role, source);
      flags |= flag;
    }
  }

  if let XraySurfaceDeclaration::Scripted { program, .. } = &descriptor.declaration
    && let Some((_, [is_reflecting, is_specular, is_transparent, is_foamed])) =
      ANOMALY_PROGRAMS.iter().find(|(name, _)| name == program)
  {
    for (is, flag) in [
      (true, StaticSurface::IS_ANOMALY_WATER),
      (*is_reflecting, StaticSurface::IS_REFLECTING),
      (*is_specular, StaticSurface::IS_SPECULAR),
      (*is_transparent, StaticSurface::IS_TRANSPARENT),
      (*is_foamed, StaticSurface::IS_FOAMED),
    ] {
      if is {
        flags |= flag;
      }
    }
  }

  StaticSurface {
    tiling: 1.0,
    detail_scale: 1.0,
    alpha_reference: 0.0,
    slice: 0.0,
    color,
    flags,
    textures: texture_slots,
  }
}
