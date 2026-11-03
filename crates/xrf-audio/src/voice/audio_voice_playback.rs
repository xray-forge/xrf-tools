use crate::voice::audio_voice_state::AudioVoiceState;

/// Where a voice is: its state and the source frame playing, as the audio thread last reported them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioVoicePlayback {
  pub state: AudioVoiceState,
  /// The source frame playing now, wrapped into the sound for a looping one.
  pub frame: u64,
}
