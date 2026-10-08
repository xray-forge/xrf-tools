use glam::Vec2;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_view_options::RenderViewOptions;

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
}

impl FogScatteringUniform {
  /// What a view scatters a frame `size` texels across with, or none: the engine's fog asked for, a scattering of none,
  /// no fog drawn (`is_fogged`), unlit or wireframe.
  pub fn for_view(options: &RenderViewOptions, is_fogged: bool, size: (u32, u32), time: f32) -> Option<[Self; 3]> {
    let fog = &options.features.fog;

    (is_fogged && options.show.is_fogged && options.mode.is_lit && !options.mode.is_wireframe && fog.is_scattered())
      .then(|| Self::stages(size, fog.scattering, time))
  }

  /// The quarter-size blur, the half-size blur, then the scattering at the frame's own size.
  pub fn stages((width, height): (u32, u32), intensity: f32, time: f32) -> [Self; 3] {
    [4, 2, 1].map(|ratio: u32| Self {
      size: Vec2::new(width.div_ceil(ratio) as f32, height.div_ceil(ratio) as f32),
      intensity: intensity.clamp(0.0, 1.0),
      time,
    })
  }
}
