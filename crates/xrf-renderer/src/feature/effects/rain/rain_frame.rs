use glam::{Vec2, Vec4};

use crate::contract::feature::effects::render_rain_drops_mode::RenderRainDropsMode;
use crate::lighting::weather::render_rainfall::RenderRainfall;
use crate::lighting::wind::ambient_gust::AmbientGust;
use crate::lighting::wind::render_wind::RenderWind;

/// What one frame's rain falls by: the weather's rainfall and the drops it is drawn with, the wind and its gusts, where
/// its cover stands, where the drops' square stands from the eye, and the splash model's indices.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RainFrame<'a> {
  pub rain: &'a RenderRainfall,
  pub drops: RenderRainDropsMode,
  pub wind: RenderWind,
  pub gust: AmbientGust,
  /// The rain's cover as its window and its shape.
  pub window: Vec4,
  pub cover_shape: Vec4,
  pub aim: Vec2,
  pub splash_indices: u32,
}
