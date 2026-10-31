use glam::Vec2;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_fog::RenderFog;

/// What a stage of the fog scattering reads, as `shaders/frame/fog_scattering.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "FogScattering")]
pub struct FogScatteringUniform {
  /// The target the stage draws into, in texels.
  pub size: Vec2,
  /// How readily the frame's bright parts are blurred into the fog.
  pub intensity: f32,
  /// The clock the blur's debanding noise turns with, in seconds.
  pub time: f32,
  /// Whether the stage reads the scene as it is held, which it takes as it is shown: the first blur.
  pub is_reading_scene: u32,
  pub _pad: u32,
}

impl FogScatteringUniform {
  /// What a view scatters a frame `size` texels across with, or none: the engine's fog asked for, a scattering of none,
  /// no weather's `fog` drawn, unlit or wireframe.
  pub fn for_view(
    options: &RenderViewOptions,
    fog: Option<&RenderFog>,
    size: (u32, u32),
    time: f32,
  ) -> Option<[Self; 3]> {
    let settings = &options.features.fog;

    (fog.is_some()
      && options.show.is_fogged
      && options.mode.is_lit
      && !options.mode.is_wireframe
      && settings.is_scattered())
    .then(|| Self::stages(size, settings.scattering, time))
  }

  /// The quarter-size blur, the half-size blur, then the scattering at the frame's own size; the first blur reads the
  /// scene, so the blurs hold it as shown and their eight bits clip nothing the tonemap would have kept.
  pub fn stages((width, height): (u32, u32), intensity: f32, time: f32) -> [Self; 3] {
    [4, 2, 1].map(|ratio: u32| Self {
      size: Vec2::new(width.div_ceil(ratio) as f32, height.div_ceil(ratio) as f32),
      intensity: intensity.clamp(0.0, 1.0),
      time,
      is_reading_scene: u32::from(ratio == 4),
      _pad: 0,
    })
  }
}
