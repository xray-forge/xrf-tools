use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_enhanced_bloom_settings::RenderEnhancedBloomSettings;
use crate::contract::render_view_options::RenderViewOptions;

/// What a stage of the enhanced bloom reads, as `shaders/frame/enhanced_bloom.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "EnhancedBloom")]
pub struct EnhancedBloomUniform {
  /// The target the stage draws into, then the frame, in texels.
  pub size: Vec4,
  /// A blur's way, one target's texel along it; then how many target texels apart a halving's or a doubling's reads
  /// spread.
  pub spread: Vec4,
  /// The threshold, the exposure, the sky's share and the vibrance.
  pub strengths: Vec4,
}

impl EnhancedBloomUniform {
  /// Halvings below half the frame's size the bloom blurs down through and back up.
  pub const LEVELS: usize = 6;

  /// What a view blooms with in the engine's place, or none: the engine's asked for, unlit or wireframe.
  pub fn for_view(options: &RenderViewOptions) -> Option<RenderEnhancedBloomSettings> {
    Some(options.features.enhanced_bloom)
      .filter(|it| it.is_enhanced() && options.mode.is_lit && !options.mode.is_wireframe)
  }

  /// A stage drawing into a target `target` texels across, for a frame `frame` texels across.
  pub fn new(target: (u32, u32), frame: (u32, u32), settings: &RenderEnhancedBloomSettings) -> Self {
    Self {
      size: Vec4::new(target.0 as f32, target.1 as f32, frame.0 as f32, frame.1 as f32),
      spread: Vec4::new(0.0, 0.0, settings.blur.max(0.0), 0.0),
      // todo: take the threshold, exposure and sky's share from the weather's `bloom_*` keys on asking, as the game
      // does when its weather presets drive the bloom.
      strengths: Vec4::new(
        settings.threshold.max(0.0),
        settings.exposure.max(0.0),
        settings.sky.max(0.0),
        settings.vibrance.max(0.0),
      ),
    }
  }

  /// The same, blurring along `x` or `y` by a target's texel.
  pub fn along(self, is_across: bool) -> Self {
    let (width, height) = (self.size.x, self.size.y);

    Self {
      spread: if is_across {
        Vec4::new(1.0 / width, 0.0, self.spread.z, 0.0)
      } else {
        Vec4::new(0.0, 1.0 / height, self.spread.z, 0.0)
      },
      ..self
    }
  }

  /// The size of a frame `(width, height)` halved `level` times, never under a texel: one is half its size.
  pub fn get_level_size((width, height): (u32, u32), level: u32) -> (u32, u32) {
    let ratio: u32 = 1 << level;

    (width.div_ceil(ratio).max(1), height.div_ceil(ratio).max(1))
  }
}
