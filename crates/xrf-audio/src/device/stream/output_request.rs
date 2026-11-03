use crate::device::audio_output_choice::AudioOutputChoice;

/// What the output's owner asks of its thread.
#[derive(Debug)]
pub(crate) enum OutputRequest {
  Choose(AudioOutputChoice),
  Close,
}
