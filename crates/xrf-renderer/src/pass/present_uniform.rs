use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_grading_settings::RenderGradingSettings;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::display_mode::DisplayMode;
use crate::pass::exposure_metering::ExposureMetering;

/// What the present pass shows and the overlays drawn over it read, as WGSL's `Present`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Present")]
pub struct PresentUniform {
  /// The debug view's index, zero for the finished scene.
  pub view: u32,
  /// One where the screen's occlusion was searched this frame.
  pub is_occluded: u32,
  /// One where the frame shown is the upscaled one.
  pub is_upscaled: u32,
  /// How far the distortion target moves the scene, a share of the screen; zero where nothing wrote it this frame.
  pub distortion: f32,
  /// The viewport's top left corner in the window, and its size, in pixels.
  pub origin: Vec2,
  pub size: Vec2,
  /// `img_corrections`' exposure, gamma and saturation, then its grading colour.
  pub corrections: Vec4,
  pub grading: Vec4,
  /// The colour a selection is outlined in, `w` one while something is selected.
  pub selection: Vec4,
  /// `PresentUniform::BLOOM_ENGINE` where the frame blooms by the engine's bloom, which the present adds where it reads
  /// the scene; `PresentUniform::BLOOM_ENHANCED` by the enhanced bloom, which it screens over the frame; zero for none.
  pub is_bloomed: u32,
  /// One where the indirect light was gathered this frame.
  pub is_indirect: u32,
  /// One where the reflections were traced this frame, then the frame's pixels a traced pixel stands for each way.
  pub is_reflected: u32,
  pub reflection_ratio: f32,
  /// The sky debanding's passes, the pixels it reads out to at most and the clock its noise turns with; no passes where
  /// the sky is drawn as it is.
  pub deband: Vec4,
  /// How the scene is shown before the rest is laid over it (`DisplayMode`), as `common/display_transform` numbers it,
  /// then the physical light's grading before its transform: its contrast, its saturation by the transform's own, and
  /// the transform's lift at the frame's key and bright end (`DisplayMode::get_lift`).
  pub display: Vec4,
  /// What the debug views read: the metres from the camera the grass's cards cast the sun's shadow to, none where they
  /// cast none, which the cascades' view marks; then the light's scale and one where it is summed, by which the
  /// reflections' view shows what the rays met as the display value combine adds it.
  pub inspect: Vec4,
}

impl PresentUniform {
  /// `is_bloomed` for the engine's bloom and for the enhanced one.
  pub const BLOOM_ENGINE: u32 = 1;
  pub const BLOOM_ENHANCED: u32 = 2;

  pub fn new(
    view: RenderDebugView,
    (is_occluded, is_indirect, reflection_ratio): (bool, bool, Option<u32>),
    is_upscaled: bool,
    distortion: f32,
    output: RenderRect,
    corrections: &RenderImageCorrections,
    (selection, is_bloomed): (Option<[f32; 3]>, bool),
  ) -> Self {
    let [r, g, b] = corrections.grading;

    Self {
      corrections: Vec4::new(corrections.exposure, corrections.gamma, corrections.saturation, 0.0),
      grading: Vec4::new(r, g, b, 0.0),
      view: view.get_index(),
      is_occluded: u32::from(is_occluded),
      is_upscaled: u32::from(is_upscaled),
      distortion,
      origin: Vec2::new(output.x as f32, output.y as f32),
      size: Vec2::new(output.width as f32, output.height as f32),
      selection: selection.map_or(Vec4::ZERO, |[r, g, b]| Vec4::new(r, g, b, 1.0)),
      is_bloomed: if is_bloomed { Self::BLOOM_ENGINE } else { 0 },
      is_indirect: u32::from(is_indirect),
      is_reflected: u32::from(reflection_ratio.is_some()),
      reflection_ratio: reflection_ratio.unwrap_or(1) as f32,
      deband: Vec4::ZERO,
      display: Vec4::ZERO,
      inspect: Vec4::ZERO,
    }
  }

  /// How many times a view averages its sky's pixels, or none: the engine's sky asked for, no sky shown, unlit or
  /// wireframe.
  pub fn get_debanding(options: &RenderViewOptions) -> Option<u32> {
    let debanding = &options.features.debanding;
    let is_shown: bool = options.mode.is_lit && !options.mode.is_wireframe && options.show.is_sky_visible;

    (is_shown && debanding.is_drawn()).then(|| debanding.get_passes())
  }

  /// The same showing its scene by `mode`, graded by `grading` where the light is physical, its transform lifted by the
  /// frame's exposed key and bright end as the exposure `metering` read them.
  pub fn with_display(self, mode: DisplayMode, grading: &RenderGradingSettings, metering: &ExposureMetering) -> Self {
    Self {
      display: Vec4::new(
        mode.get_index() as f32,
        grading.contrast,
        grading.saturation,
        mode.get_lift((metering.key, metering.bright), grading.contrast),
      ),
      ..self
    }
  }

  /// The same, the cascades' view striping the grass's cards' reach of the sun's shadow (`reach` metres), none where
  /// they cast none.
  pub fn with_grass_reach(self, reach: Option<f32>) -> Self {
    Self {
      inspect: Vec4::new(reach.unwrap_or(0.0), self.inspect.y, self.inspect.z, 0.0),
      ..self
    }
  }

  /// The same, the reflections' view showing what the rays met as the display value combine adds it: the light by the
  /// light's `scale`, encoded where the light is summed (`is_summed`).
  pub fn with_reflection_light(self, scale: f32, is_summed: bool) -> Self {
    Self {
      inspect: Vec4::new(self.inspect.x, scale, f32::from(u8::from(is_summed)), 0.0),
      ..self
    }
  }

  /// The same, the enhanced bloom screened over the frame in place of the engine's.
  pub fn with_enhanced_bloom(self) -> Self {
    Self {
      is_bloomed: Self::BLOOM_ENHANCED,
      ..self
    }
  }

  /// The same, the sky debanded over `passes` passes out to `radius` pixels, its noise turning with `time` seconds.
  pub fn with_debanding(self, passes: u32, radius: f32, time: f32) -> Self {
    Self {
      deband: Vec4::new(passes as f32, radius, time, 0.0),
      ..self
    }
  }

  /// The same drawn into a screenshot `size` pixels across, apart from the window and without the selection's outline.
  pub fn for_screenshot(self, (width, height): (u32, u32)) -> Self {
    Self {
      origin: Vec2::ZERO,
      size: Vec2::new(width as f32, height as f32),
      selection: Vec4::ZERO,
      ..self
    }
  }
}
