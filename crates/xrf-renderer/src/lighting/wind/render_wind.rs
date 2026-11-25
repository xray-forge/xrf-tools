/// Metres a second of the strongest wind, at which what a wind drives is at its most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const RENDER_STRONGEST_WIND: f32 = 20.0;

/// The weather's wind, which moves the foliage, leans the rain and drives the enhanced water's waves and the wet's
/// gusts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderWind {
  /// `wind_direction`, in radians, the way it blows in engine space.
  pub direction: f32,
  /// Metres a second.
  pub speed: f32,
}

impl RenderWind {
  /// The share of its strongest this wind blows at, from nothing to one.
  pub fn get_strength(&self) -> f32 {
    Self::get_strength_of(self.speed)
  }

  /// The share of its strongest a wind of `speed` metres a second blows at.
  pub fn get_strength_of(speed: f32) -> f32 {
    (speed / RENDER_STRONGEST_WIND).clamp(0.0, 1.0)
  }
}
