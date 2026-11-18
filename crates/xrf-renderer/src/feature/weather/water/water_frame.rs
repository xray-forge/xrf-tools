use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::frame::targets::view_targets::ViewTargets;
use crate::lighting::wind::render_wind::RenderWind;
use crate::scene::level::view::view_time::ViewTime;

/// What a level view's water reads of its frame beside its own state.
pub(crate) struct WaterFrame<'a> {
  pub targets: Option<&'a ViewTargets>,
  /// The tiles of the frame the level's water in sight may touch, by the spheres about it; none where none is.
  pub tiles: ScreenTiles,
  /// The weather's `water_intensity`.
  pub intensity: f32,
  pub wind: RenderWind,
  /// The rain's density.
  pub rain: f32,
  pub time: ViewTime,
}
