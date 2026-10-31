use std::time::Duration;

/// An ambient a keyframe names by `ambient`, `CEnvAmbient`: the effects it picks among near the camera and how long it
/// waits between them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldAmbient {
  /// Its effects, by their sections' names; one naming none plays none.
  pub effects: Vec<String>,
  /// The least it waits from one effect's start to the next, `m_effect_period.x`.
  pub least_period: Duration,
  /// The most it waits from one effect's start to the next, `m_effect_period.y`.
  pub most_period: Duration,
}
