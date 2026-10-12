use crate::frame::view_targets::ViewTargets;
use crate::lighting::render_wind::RenderWind;

/// What a level view's water reads of its frame beside its own state: the targets, the weather's
/// `water_intensity`, wind and rain density, and the clock.
pub struct WaterFrame<'a> {
  pub targets: Option<&'a ViewTargets>,
  /// Whether any of the level's water may be in sight.
  pub is_in_view: bool,
  pub intensity: f32,
  pub wind: RenderWind,
  pub rain: f32,
  pub time: f32,
}
