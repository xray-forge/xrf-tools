use glam::Vec4;

use crate::feature::page::present_marks::PresentMarks;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::pass::frame_lighting::light_encoding::LightEncoding;

/// What the other units settled this frame that the window pass reads.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PresentInputs {
  pub is_upscaled: bool,
  pub is_occluded: bool,
  pub is_indirect: bool,
  /// Frame pixels a traced reflection pixel stands for each way, where reflections are traced.
  pub reflection_ratio: Option<u32>,
  /// The tiles anything distorts the frame in.
  pub distortion: ScreenTiles,
  pub selection_color: Option<[f32; 3]>,
  pub marks: PresentMarks,
  pub is_engine_bloomed: bool,
  pub is_enhanced_bloomed: bool,
  /// Whether the engine's bloom is built this frame, shown or under the enhanced one; the distortion blurs towards it.
  pub is_engine_built: bool,
  /// Metres from the camera the grass's cards cast the sun's shadow to, none where they cast none.
  pub grass_reach: Option<f32>,
  pub light: LightEncoding,
  /// How the frame is shown (`DisplayMode::get_display`), its transform lifted by what the exposure metered.
  pub display: Vec4,
  /// The frame the film's grains are drawn for.
  pub grain_frame: u32,
}
