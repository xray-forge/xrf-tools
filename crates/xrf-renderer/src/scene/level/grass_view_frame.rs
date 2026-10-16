use crate::camera::camera_view::CameraView;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;

/// What a view's grass plants and draws by this frame beside its settings: the camera, the static cull's discard, the
/// wind, the puddles no tuft is planted in, and how its cards cast shadows.
#[derive(Clone, Copy)]
pub struct GrassViewFrame<'a> {
  pub camera: &'a CameraView,
  /// `r_ssaDISCARD` as the static cull compares it.
  pub discard_below: f32,
  /// Seconds the wind blows by.
  pub time: f32,
  /// Whether the weather blows the wind at all.
  pub is_windy: bool,
  /// The wind's strength, `wind_strength_factor`.
  pub strength: f32,
  pub foliage: &'a FoliageWindValues,
  /// Where the puddles stood, and the window they stand in.
  pub puddles: (&'a wgpu::TextureView, GrassPuddlesUniform),
  /// Whether the view traces its light, which traces the cards' shadows.
  pub is_traced: bool,
  /// How far the sun's cascades reach where they draw the cards, none where they do not.
  pub cascade_reach: Option<f32>,
}
