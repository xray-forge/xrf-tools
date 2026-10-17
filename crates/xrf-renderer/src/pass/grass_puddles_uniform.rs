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
  /// The level surface's `window` as `LevelSurface::get_window` gives it, with puddles drawn or not.
  pub fn new((surface, shape): (Vec4, Vec4), is_drawn: bool) -> Self {
    Self {
      surface,
      shape,
      state: Vec4::new(f32::from(u8::from(is_drawn)), 0.0, 0.0, 0.0),
    }
  }
}
