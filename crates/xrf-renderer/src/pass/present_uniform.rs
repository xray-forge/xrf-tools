use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_debanding_quality::RenderDebandingQuality;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;

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
  /// One where the frame blooms by the engine's bloom, which the present adds where it reads the scene; two by the
  /// enhanced bloom, which it screens over the frame.
  pub is_bloomed: u32,
  /// One where the indirect light was gathered this frame.
  pub is_indirect: u32,
  /// One where the reflections were traced this frame, then the frame's pixels a traced pixel stands for each way.
  pub is_reflected: u32,
  pub reflection_ratio: f32,
  /// The sky debanding's passes, the pixels it reads out to at most and the clock its noise turns with; no passes where
  /// the sky is drawn as it is.
  pub deband: Vec4,
}

impl PresentUniform {
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
      is_bloomed: u32::from(is_bloomed),
      is_indirect: u32::from(is_indirect),
      is_reflected: u32::from(reflection_ratio.is_some()),
      reflection_ratio: reflection_ratio.unwrap_or(1) as f32,
      deband: Vec4::ZERO,
    }
  }

  /// How hard a view debands its sky, or none: the engine's sky asked for, no sky shown, unlit or wireframe.
  pub fn get_debanding(options: &RenderViewOptions) -> Option<RenderDebandingQuality> {
    let debanding = &options.features.debanding;
    let is_shown: bool = options.mode.is_lit && !options.mode.is_wireframe && options.show.is_sky_visible;

    (is_shown && debanding.is_drawn()).then_some(debanding.quality)
  }

  /// The same, the enhanced bloom screened over the frame in place of the engine's.
  pub fn with_enhanced_bloom(self) -> Self {
    Self { is_bloomed: 2, ..self }
  }

  /// The same, the sky debanded over `passes` passes out to `radius` pixels, its noise turning with `time` seconds.
  pub fn with_debanding(self, passes: u32, radius: f32, time: f32) -> Self {
    Self {
      deband: Vec4::new(passes as f32, radius, time, 0.0),
      ..self
    }
  }
}
