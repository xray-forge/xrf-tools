use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::scene::level::level_flares::LevelFlares;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;

/// What a sky's bind group binds: the weather textures' generation, the references of its seven slots (the two skies,
/// the two environments, the two cloud layers and the sun's sprite), and how many environment cubes.
pub(crate) type SkyGroupKey = (u64, [Option<String>; 7], u64);

/// What draws a level in one view, and what that keeps from frame to frame: the uniform buffers its passes read, the
/// bind groups made for them with what each was made from, and the effects drawn from the view (the sun's shadows, the
/// rain cover, the flares).
pub struct SceneRenderer {
  pub temporal_uniform: wgpu::Buffer,
  pub fsr_uniform: wgpu::Buffer,
  pub rain_cover: RainCover,
  /// The sun's sprite, lens flares and gradient.
  pub flares: LevelFlares,
  /// The splash's model, with the level's weather it was built for.
  pub splash: Option<(usize, WeatherModelBuffers)>,
  /// Every bolt model of the level's weather, with the weather they were built for, and an empty one the glows bind.
  pub thunder_models: Option<(usize, Vec<WeatherModelBuffers>)>,
  pub no_model: WeatherModelBuffers,
  pub shadows: LevelShadows,
}

impl SceneRenderer {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    let uniform = |label: &str, size: usize| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    Self {
      rain_cover: RainCover::new(device, view_layout, args_size),
      temporal_uniform: uniform("temporal", size_of::<TemporalUniform>()),
      fsr_uniform: uniform("fsr2", size_of::<FsrUniform>()),
      flares: LevelFlares::new(device),
      splash: None,
      thunder_models: None,
      no_model: WeatherModelBuffers::new(device, None),
      shadows: LevelShadows::new(device),
    }
  }
}
