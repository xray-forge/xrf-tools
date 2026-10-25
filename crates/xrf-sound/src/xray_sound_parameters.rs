#[derive(Clone, Debug, PartialEq)]
pub struct XRaySoundParameters {
  pub min_distance: f32,
  pub max_distance: f32,
  pub base_volume: f32,
  pub game_type: u32,
  pub max_ai_distance: f32,
}
