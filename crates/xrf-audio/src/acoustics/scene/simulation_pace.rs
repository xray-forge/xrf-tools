/// How a scene's acoustics simulation keeps up with it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SimulationPace {
  /// On a thread of its own as fast as the wall clock allows, the control side taking what it last heard: what plays
  /// live.
  #[default]
  RealTime,
  /// Inline on the control side as it ticks, at the same cadence on the scene's own clock, so what is heard depends on
  /// the scene's frames alone: what an offline render repeats.
  Stepped,
}
