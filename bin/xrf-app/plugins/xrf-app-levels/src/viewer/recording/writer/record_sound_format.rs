/// A recording's sound: the device's sample rate, resampled to 48 kHz unless AAC takes it (44.1 or 48 kHz), and one or
/// two channels; a resampled rate needs two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RecordSoundFormat {
  pub rate: u32,
  pub channels: u16,
}
