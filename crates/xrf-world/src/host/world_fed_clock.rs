/// The game's clock as its host runs it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldFedClock {
  /// Game seconds since midnight.
  pub time: f32,
  /// Game seconds a real second: the rate the renderer paces the weather's effects by, a lens flare's fade among them.
  pub rate: f32,
}
