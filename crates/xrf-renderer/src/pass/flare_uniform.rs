use glam::{Vec3, Vec4};

use crate::host::render_lens_flare::RenderLensFlare;

/// Flares `shaders/common/flares.wgsl` holds room for; a section with more draws its first ones.
pub const FLARE_SLOTS: usize = 16;

/// What `shaders/common/flares.wgsl` reads as its `Flares`: where the sun stands, its colour and the lens flare's.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlareUniform {
  pub to_sun: Vec4,
  pub sun: Vec4,
  pub color: Vec4,
  pub gradient: Vec4,
  pub flares: [Vec4; FLARE_SLOTS],
}

impl FlareUniform {
  /// One frame's lens flare: towards the sun in view space and in the world, the sun's colour, how far the flare has
  /// faded in, and the seconds since the last frame its visibility eases over.
  pub fn new(flare: &RenderLensFlare, (to_sun, toward_sun): (Vec3, Vec3), color: Vec3, faded: f32, delta: f32) -> Self {
    let mut flares: [Vec4; FLARE_SLOTS] = [Vec4::ZERO; FLARE_SLOTS];

    for (slot, it) in flares.iter_mut().zip(&flare.flares) {
      *slot = Vec4::new(it.position, it.radius, it.opacity, 0.0);
    }

    Self {
      to_sun: to_sun.extend(delta),
      sun: toward_sun.extend(faded),
      color: color.extend(flare.flares.len().min(FLARE_SLOTS) as f32),
      gradient: flare
        .gradient
        .as_ref()
        .map_or(Vec4::ZERO, |it| Vec4::new(it.radius, it.opacity, 1.0, 0.0)),
      flares,
    }
  }
}
