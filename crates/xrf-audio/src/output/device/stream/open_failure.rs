use crate::mixer::audio_mixer::AudioMixer;

/// A stream that could not open, with the mixer it was given back, unless the device kept it.
pub(crate) struct OpenFailure {
  pub(crate) mixer: Option<AudioMixer>,
  pub(crate) message: String,
}
