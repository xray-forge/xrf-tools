use glam::{Vec2, Vec4};

use crate::contract::render_rain_drops_mode::RenderRainDropsMode;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::render_wind::RenderWind;

/// What one frame's rain falls by: the weather's rainfall and the drops it is drawn with, the wind and its strength,
/// where its cover stands, where the enhanced drops' square stands from the eye, and the splash model's indices.
#[derive(Clone, Copy, Debug)]
pub struct RainFrame<'a> {
  pub rain: &'a RenderRainfall,
  pub drops: RenderRainDropsMode,
  pub wind: RenderWind,
  /// `wind_strength_factor`: a tenth of it leans the engine's streaks with the wind (`CEffect_Rain::Born`), and its
  /// swing about a half stands in for the gust Monolith's `CEffect_Rain::Prepare` leans its drops by.
  pub strength: f32,
  /// The rain's cover as its window and its shape.
  pub window: Vec4,
  pub cover_shape: Vec4,
  pub aim: Vec2,
  pub splash_indices: u32,
}
