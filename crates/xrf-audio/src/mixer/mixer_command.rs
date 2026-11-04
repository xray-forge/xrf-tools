use crate::mixer::audio_bus::AudioBus;
use crate::mixer::tap::mixer_tap::MixerTap;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::voice::Voice;

/// What the control side asks of the audio thread, taken at the start of each buffer: a voice's life, the mix's levels
/// and its tap. A voice's continuous controls go through its slot's parameter block instead.
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
  /// A tap to push the mix into from the next block, in place of any the mixer holds, which goes back to be freed.
  SetTap(Box<MixerTap>),
}
