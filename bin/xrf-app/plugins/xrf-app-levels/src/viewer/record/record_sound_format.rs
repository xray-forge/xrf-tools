/// A recording's sound: its frames a second, 44.1 or 48 kHz as AAC takes, and its channels, one or two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordSoundFormat {
  pub rate: u32,
  pub channels: u16,
}
