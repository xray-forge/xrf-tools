use crate::mixer::audio_bus::AudioBus;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::voice::Voice;

/// What the control side asks of the audio thread, taken at the start of each buffer: a voice's life and the mix's
/// levels. A voice's continuous controls go through its slot's parameter block instead.
#[derive(Debug)]
pub(crate) enum MixerCommand {
  /// A voice to place in its slot, built and boxed on the control thread.
  Start(Box<Voice>),
  Stop(AudioVoiceId),
  Pause(AudioVoiceId),
  Resume(AudioVoiceId),
  SetBusGain {
    bus: AudioBus,
    gain: f32,
  },
  SetMasterGain(f32),
}
