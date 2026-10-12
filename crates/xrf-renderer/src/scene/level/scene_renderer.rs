use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_surface::LevelSurface;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::surroundings_cube::SurroundingsCube;

/// What a sky's bind group binds: the weather textures' generation, the references of its seven slots (the two skies,
/// the two environments, the two cloud layers and the sun's sprite), and how many environment cubes.
pub(crate) type SkyGroupKey = (u64, [Option<String>; 7], u64);

/// What a view draws of the level from views of its own, kept from frame to frame: the sun's shadows, the rain cover
/// and the level's surface seen from overhead, and the level around the camera as a cube.
pub struct SceneRenderer {
  pub rain_cover: RainCover,
  /// The level's surface around the camera without its trees, which the puddles are seated on.
  pub level_surface: LevelSurface,
  /// The level around the camera as a cube, a face a frame, which a reflection meeting nothing on the screen reads.
  pub surroundings: SurroundingsCube,
  pub shadows: LevelShadows,
}

impl SceneRenderer {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    Self {
      rain_cover: RainCover::new(device, view_layout, args_size),
      level_surface: LevelSurface::new(device, view_layout, args_size),
      surroundings: SurroundingsCube::new(device, view_layout, args_size),
      shadows: LevelShadows::new(device),
    }
  }
}
