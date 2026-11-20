/// A sound the scene's walls may stand in front of, as the simulation is handed it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SimulatedSource {
  pub(crate) id: u64,
  /// In the game's space.
  pub(crate) position: [f32; 3],
}
