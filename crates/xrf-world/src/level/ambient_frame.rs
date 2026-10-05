use xrf_renderer::{RenderAmbients, RenderLevelWeather};

/// What the weather's ambient effects read of a frame's weather: the ambients its keyframes name, and every ambient and
/// effect its level plays with.
#[derive(Clone, Copy)]
pub struct AmbientFrame<'a> {
  pub ambients: &'a RenderAmbients,
  pub level: &'a RenderLevelWeather,
}
