/// The ground and the sound of the player's last step, which the next step on the same ground does not repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GameLastStep {
  /// The ground's game material, by its id.
  pub ground: u16,
  /// Which of the ground's step sounds played.
  pub sound: usize,
}
