use xrf_collections::rtrb::Consumer;

use crate::mixer::audio_mixer::AudioMixer;

/// A stream that could not open, with the mixer it was given back, or, where the device keeps it past the wait, the
/// ring its callback hands it back through later.
pub(crate) struct OpenFailure {
  pub(crate) mixer: Result<AudioMixer, Consumer<AudioMixer>>,
  pub(crate) message: String,
}
