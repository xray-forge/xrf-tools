use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::bloom_pass::BloomGroups;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::view_light_groups::ViewLightGroups;
use crate::pass::wet_uniform::WetUniform;
use crate::scene::level::level_flares::LevelFlares;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_water::LevelWater;
use crate::scene::level::rain_cover::RainCover;

/// What a sky's bind group binds: the weather textures' generation, the references of its seven slots (the two skies,
/// the two environments, the two cloud layers and the sun's sprite), and how many environment cubes.
pub(crate) type SkyGroupKey = (u64, [Option<String>; 7], u64);

/// What draws a level in one view, and what that keeps from frame to frame: the uniform buffers its passes read, the
/// bind groups made for them with what each was made from, and the effects drawn from the view (the water, the sun's
/// shadows, the rain cover, the flares).
pub struct SceneRenderer {
  pub cull_params: wgpu::Buffer,
  pub occlusion: wgpu::Buffer,
  pub lighting: wgpu::Buffer,
  /// The cull's and the draws' bind groups, with the scene generation (and the cull, the targets epoch) they bind.
  pub cull_group: Option<((u64, u64), wgpu::BindGroup)>,
  pub draw_groups: Option<(u64, StaticDrawGroups)>,
  /// The lighting passes' bind groups, made again with the targets, and the shadow maps' epoch they bind.
  pub light_groups: Option<(u64, ViewLightGroups)>,
  /// The sky's textures as bound, with the cache's generation and the references they bind.
  pub sky_group: Option<(SkyGroupKey, wgpu::BindGroup)>,
  /// Bumped whenever the sky's bind group is made again, which the water's follows.
  pub sky_version: u64,
  /// The water, its uniform, and what its enhanced kind reads and draws first.
  pub water: LevelWater,
  /// What the present pass shows, a [`PresentUniform`].
  pub present: wgpu::Buffer,
  pub temporal_uniform: wgpu::Buffer,
  pub fsr_uniform: wgpu::Buffer,
  pub upscale_uniform: wgpu::Buffer,
  /// The present pass's bind group, with the targets' and the upscale's epochs and the frame it shows.
  pub present_group: Option<((u64, u64, usize), wgpu::BindGroup)>,
  /// The sorted composited clusters' bind group, with the scene generation and the list's epoch it binds.
  pub sorted_group: Option<((u64, u64), wgpu::BindGroup)>,
  /// The overlay pass's bind group, with the targets' epoch it binds.
  pub overlay_group: Option<(u64, wgpu::BindGroup)>,
  pub rain_cover: RainCover,
  pub rain: wgpu::Buffer,
  /// The rain's bind group, with the weather textures' generation and the splash's weather it binds.
  pub rain_group: Option<((u64, usize), wgpu::BindGroup)>,
  pub wet: wgpu::Buffer,
  /// The wet surfaces' bind groups, with the targets' epoch and the weather textures' generation they bind.
  pub wet_groups: Option<((u64, u64), [wgpu::BindGroup; 2])>,
  pub thunder: wgpu::Buffer,
  /// A strike's bind groups, with the weather textures' generation, the weather and the bolt they bind.
  pub thunder_groups: Option<((u64, usize, String), [wgpu::BindGroup; 3])>,
  /// The sun's sprite, lens flares and gradient.
  pub flares: LevelFlares,
  /// What the bloom's build and its two blurs read, and what they draw with, at the targets' epoch.
  pub bloom_uniforms: [wgpu::Buffer; 3],
  pub bloom_groups: Option<(u64, BloomGroups)>,
  pub shadows: LevelShadows,
  pub occlusion_uniform: wgpu::Buffer,
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
      cull_params: uniform("static cull", size_of::<StaticCullParams>()),
      occlusion: uniform("static occlusion", size_of::<StaticOcclusionUniform>()),
      lighting: uniform("lighting", size_of::<LightingUniform>()),
      cull_group: None,
      draw_groups: None,
      light_groups: None,
      sky_group: None,
      sky_version: 0,
      water: LevelWater::new(device),
      present: uniform("present", size_of::<PresentUniform>()),
      temporal_uniform: uniform("temporal", size_of::<TemporalUniform>()),
      fsr_uniform: uniform("fsr2", size_of::<FsrUniform>()),
      upscale_uniform: uniform("upscale", size_of::<UpscaleUniform>()),
      present_group: None,
      overlay_group: None,
      sorted_group: None,
      rain: uniform("rain", size_of::<RainUniform>()),
      rain_group: None,
      wet: uniform("wet", size_of::<WetUniform>()),
      wet_groups: None,
      thunder: uniform("thunder", size_of::<ThunderUniform>()),
      thunder_groups: None,
      flares: LevelFlares::new(device),
      bloom_uniforms: ["bloom build", "bloom across", "bloom down"]
        .map(|label| uniform(label, size_of::<BloomUniform>())),
      bloom_groups: None,
      shadows: LevelShadows::new(device),
      occlusion_uniform: uniform("ambient occlusion", size_of::<AmbientOcclusionUniform>()),
    }
  }
}
