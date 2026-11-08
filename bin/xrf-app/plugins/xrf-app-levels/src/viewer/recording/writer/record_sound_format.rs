/// A recording's sound: its frames a second, a device's, which a rate other than the 44.1 or 48 kHz AAC takes is
/// brought to 48 kHz from; and its channels, one or two, two at a rate brought to 48 kHz.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RecordSoundFormat {
  pub rate: u32,
  pub channels: u16,
}
