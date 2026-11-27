/// What presentation reads after a tick: the tick and the game state's own frame, shared whole between the loop and
/// whoever reads it. The one way state leaves the simulation besides its events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopFrame<F> {
  pub tick: u64,
  pub state: F,
}
