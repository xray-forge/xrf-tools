use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// Where the puddles stand, as the grass's planting reads them (`shaders/grass/puddles.wgsl`) to plant no tuft in a
/// puddle's water: the level surface's window (its centre in `x` and `z`, its half width, the height it is seen from),
/// its shape (texels across, metres down, a site cell's metres), and one where puddles are drawn.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "GrassPuddles")]
pub struct GrassPuddlesUniform {
  pub surface: Vec4,
  pub shape: Vec4,
  pub state: Vec4,
}

impl GrassPuddlesUniform {
  /// The level surface's window as `LevelSurface::get_window` gives it, its `surface` and its `shape`, with no puddles
  /// drawn.
  pub fn new(surface: Vec4, shape: Vec4) -> Self {
    Self {
      surface,
      shape,
      state: Vec4::ZERO,
    }
  }

  /// The same with the wet surfaces' puddles drawn, which the grass plants around.
  pub fn with_puddles(self) -> Self {
    Self {
      state: Vec4::new(1.0, 0.0, 0.0, 0.0),
      ..self
    }
  }
}
