use crate::pass::exposure_metering::ExposureMetering;

/// What the other units decided this frame that the window pass reads: whether an ambient occlusion and an indirect
/// light are drawn, how many frame pixels a traced reflection pixel stands for, whether anything distorts the frame,
/// the selection's colour, which bloom is laid over it, how far the grass casts the sun's shadow, the light's scale
/// and whether it is summed, which the reflections' view shows what the rays met by, what the exposure metered,
/// which the display transforms are lifted by, and the frame the camera's film grain is drawn for.
#[derive(Clone, Copy, Debug, Default)]
pub struct PresentInputs {
  pub is_occluded: bool,
  pub is_indirect: bool,
  pub reflection_ratio: Option<u32>,
  pub is_distorting: bool,
  pub selection_color: Option<[f32; 3]>,
  pub is_engine_bloomed: bool,
  pub is_enhanced_bloomed: bool,
  /// Metres from the camera the grass's cards cast the sun's shadow to, none where they cast none.
  pub grass_reach: Option<f32>,
  /// The scale the frame's light is written at (`light_scale` in `common/lighting`), and whether the light is the
  /// authored levels' summed light (`LightingUniform::summed_light`).
  pub light_scale: f32,
  pub is_light_summed: bool,
  /// What the exposure metered, whose exposed key and bright end the display transforms are lifted by.
  pub metering: ExposureMetering,
  /// The frame the film's grains are drawn for.
  pub grain_frame: u32,
}
