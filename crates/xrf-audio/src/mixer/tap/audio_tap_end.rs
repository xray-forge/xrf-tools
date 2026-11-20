use std::fmt;

/// Why a tap's stream ended before its reader let it go.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioTapEnd {
  /// The mixer went on at another rate, as a new device opened: what came before is whole, at `from` frames a second.
  RateChanged { from: u32, to: u32 },
}

impl fmt::Display for AudioTapEnd {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::RateChanged { from, to } => write!(
        formatter,
        "the audio went from {from} Hz to {to} Hz, as another device opened"
      ),
    }
  }
}
