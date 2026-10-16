/// What the other units decided this frame that the window pass reads: whether an ambient occlusion and an indirect
/// light are drawn, how many frame pixels a traced reflection pixel stands for, whether anything distorts the frame,
/// the selection's colour, which bloom is laid over it, and how far the grass casts the sun's shadow.
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
}
