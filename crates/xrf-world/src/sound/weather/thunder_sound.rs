use std::time::{Duration, Instant};

use glam::Vec3;

/// A strike that thunders, `play_no_feedback` in `CEffect_Thunderbolt::Bolt`: the bolt's section, where it struck from
/// in engine space, and how far from the camera; heard `distance / 300` seconds after it struck, over `distance / 2` to
/// `distance * 2` metres.
#[derive(Clone, Debug, PartialEq)]
pub struct ThunderSound {
  pub bolt: String,
  pub position: Vec3,
  pub distance: f32,
  /// When it struck, by the frame's clock.
  pub at: Instant,
}

impl ThunderSound {
  /// The speed sound travels at, as the engine delays it.
  pub const SPEED: f32 = 300.0;

  /// How long after the strike it is heard.
  pub fn get_delay(&self) -> Duration {
    Duration::try_from_secs_f32(self.distance / Self::SPEED).unwrap_or_default()
  }

  /// The metres it is heard within.
  pub fn get_range(&self) -> [f32; 2] {
    [self.distance / 2.0, self.distance * 2.0]
  }
}
