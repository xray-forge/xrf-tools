use std::time::Duration;

/// The blast of wind an ambient effect brings, `wind_blast_*`; the default is an effect writing none, which the engine
/// reads as no strength from straight ahead.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderWindBlast {
  /// The wind's strength it blows towards, `wind_blast_strength`.
  pub strength: f32,
  /// Which way it blows, in radians about the vertical, `wind_blast_longitude`.
  pub longitude: f32,
  /// How long it takes to rise, `wind_blast_in_time`.
  pub in_time: Duration,
  /// How long it takes to fall once the effect ends, `wind_blast_out_time`.
  pub out_time: Duration,
}
