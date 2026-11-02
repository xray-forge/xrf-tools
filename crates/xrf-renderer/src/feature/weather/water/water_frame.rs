use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::frame::targets::view_targets::ViewTargets;
use crate::lighting::wind::render_wind::RenderWind;

/// What a level view's water reads of its frame beside its own state: the targets, where its water may draw, the
/// weather's `water_intensity`, wind and rain density, and the clock.
pub struct WaterFrame<'a> {
  pub targets: Option<&'a ViewTargets>,
  /// The tiles of the frame the level's water in sight may touch, by the spheres about it; none where none is.
  pub tiles: ScreenTiles,
  pub intensity: f32,
  pub wind: RenderWind,
  pub rain: f32,
  pub time: f32,
}
