/// What the other units decided this frame that the window pass reads: whether an ambient occlusion and an indirect
/// light are drawn, how many frame pixels a traced reflection pixel stands for, whether anything distorts the frame,
/// the selection's colour, which bloom is laid over it, how far the grass casts the sun's shadow, the light's scale
/// and whether it is summed, which the reflections' view shows what the rays met by, and the key the display
/// transforms are anchored at.
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
  /// The frame's exposed key the display transforms are anchored at (`ExposureFeature::get_key`), a daylit exterior's
  /// where nothing was metered.
  pub exposed_key: f32,
}
