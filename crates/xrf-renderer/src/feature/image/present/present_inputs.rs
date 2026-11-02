use glam::Vec4;

use crate::feature::page::present_marks::PresentMarks;
use crate::pass::frame_lighting::light_encoding::LightEncoding;

/// What the other units decided this frame that the window pass reads: whether the scene was upscaled into the
/// viewport, whether an ambient occlusion and an indirect light are drawn, how many frame pixels a traced reflection
/// pixel stands for, whether anything distorts the frame, the selection's colour, the helpers' marks and fading, which
/// bloom is laid over it, how far the grass casts the sun's shadow, the light's scale and whether it is summed, which
/// the reflections' view shows what the rays met by, how the frame is shown, and the frame the camera's film grain is
/// drawn for.
#[derive(Clone, Copy, Debug, Default)]
pub struct PresentInputs {
  pub is_upscaled: bool,
  pub is_occluded: bool,
  pub is_indirect: bool,
  pub reflection_ratio: Option<u32>,
  pub is_distorting: bool,
  pub selection_color: Option<[f32; 3]>,
  pub marks: PresentMarks,
  pub is_engine_bloomed: bool,
  pub is_enhanced_bloomed: bool,
  /// Metres from the camera the grass's cards cast the sun's shadow to, none where they cast none.
  pub grass_reach: Option<f32>,
  pub light: LightEncoding,
  /// How the frame is shown (`DisplayMode::get_display`), its transform lifted by what the exposure metered.
  pub display: Vec4,
  /// The frame the film's grains are drawn for.
  pub grain_frame: u32,
}
