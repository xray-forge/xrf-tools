use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::host::render_lens_flare::RenderLensFlare;
use crate::pass::flare_frame::FlareFrame;

/// What `shaders/common/flares.wgsl` reads as its `Flares`: where the sun stands, its colour and the lens flare's.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Flares")]
pub struct FlareUniform {
  pub to_sun: Vec4,
  pub sun: Vec4,
  pub color: Vec4,
  pub gradient: Vec4,
  pub flares: [Vec4; FlareUniform::SLOTS],
}

impl FlareUniform {
  /// Flares `shaders/common/flares.wgsl` holds room for; a section with more draws its first ones.
  pub const SLOTS: usize = 16;

  /// The lens `flare` drawn in one `frame`.
  pub fn new(flare: &RenderLensFlare, frame: FlareFrame) -> Self {
    let mut flares: [Vec4; Self::SLOTS] = [Vec4::ZERO; Self::SLOTS];

    for (slot, it) in flares.iter_mut().zip(&flare.flares) {
      *slot = Vec4::new(it.position, it.radius, it.opacity, 0.0);
    }

    Self {
      to_sun: frame.to_sun.extend(frame.delta),
      sun: frame.toward_sun.extend(frame.faded),
      color: frame.color.extend(0.0),
      gradient: flare
        .gradient
        .as_ref()
        .map_or(Vec4::ZERO, |it| Vec4::new(it.radius, it.opacity, 1.0, 0.0)),
      flares,
    }
  }
}
