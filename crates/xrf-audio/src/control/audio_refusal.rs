use std::fmt::{Display, Formatter};

/// Why the audio thread was not asked to do something.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioRefusal {
  /// Every voice slot plays: the engines' target budget, here the mixer's pool.
  VoicesExhausted,
  /// The command ring is full: the audio thread has not taken commands for a while, as when no device plays.
  CommandsFull,
}

impl Display for AudioRefusal {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(match self {
      Self::VoicesExhausted => "every voice is playing",
      Self::CommandsFull => "the audio thread is not taking commands",
    })
  }
}

impl std::error::Error for AudioRefusal {}
