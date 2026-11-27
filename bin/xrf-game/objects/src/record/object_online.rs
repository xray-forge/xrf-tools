/// An object's row in the online table: it is online, near enough the actor for the systems that act on online objects
/// (the world's models, physics, AI) to take it up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectOnline {
  /// The tick it went online at.
  pub since: u64,
}
