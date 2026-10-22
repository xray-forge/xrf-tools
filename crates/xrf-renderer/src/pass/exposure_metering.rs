/// What the physical light's exposure metered, as last read back: the key the frame is shown at as its scale exposes
/// it (`ExposureHead::key`; a daylit exterior's where nothing is metered, nought under the engine's light) and its bright
/// end so exposed, then, log2, the land's average at a scale of one (the window's, the sky left out) and the sky's level
/// and bright end as the game paints them, before the scale its pictures are drawn at, each none until metered.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExposureMetering {
  pub key: f32,
  pub bright: Option<f32>,
  pub land: Option<f32>,
  pub sky: Option<f32>,
  pub sky_bright: Option<f32>,
}
