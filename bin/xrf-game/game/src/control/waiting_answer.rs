use std::time::Instant;

/// A request answered once every input it played is delivered: the last of them falls due at `due`.
#[derive(Clone, Debug)]
pub(crate) struct WaitingAnswer {
  pub id: String,
  pub due: Instant,
  /// How many events it played.
  pub events: usize,
}
