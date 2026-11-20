use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// Where the puddles stand, so the grass's planting puts no tuft in their water (`shaders/grass/puddles.wgsl`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "GrassPuddles")]
pub(crate) struct GrassPuddlesUniform {
  /// The level surface's window: its centre in `x` and `z`, its half width, the height it is seen from.
  pub surface: Vec4,
  /// The surface's texels across, metres down, and a site cell's metres.
  pub shape: Vec4,
  /// One where puddles are drawn, then the metres from the eye they are drawn to.
  pub state: Vec4,
}

impl GrassPuddlesUniform {
  /// The level surface's window as `LevelSurface::get_window` gives it, its `surface` and its `shape`, with no puddles
  /// drawn.
  pub(crate) fn new(surface: Vec4, shape: Vec4) -> Self {
    Self {
      surface,
      shape,
      state: Vec4::ZERO,
    }
  }

  /// The same with puddles drawn out to `distance` metres from the eye.
  pub(crate) fn with_puddles(self, distance: f32) -> Self {
    Self {
      state: Vec4::new(1.0, distance, 0.0, 0.0),
      ..self
    }
  }
}
