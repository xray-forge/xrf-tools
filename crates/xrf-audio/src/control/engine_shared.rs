use std::sync::Arc;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::mixer::reverb_parameters::ReverbParameters;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_status::VoiceStatus;

/// What an engine's two halves both hold: each slot's controls, written by the control side, and its report, written
/// by the audio thread; the mixer's clock and load; the acoustics; and how the listener's reverb decays.
#[derive(Clone)]
pub(crate) struct EngineShared {
  pub(crate) parameters: Arc<[VoiceParameters]>,
  pub(crate) statuses: Arc<[VoiceStatus]>,
  pub(crate) meter: AudioMixerMeter,
  pub(crate) acoustics: AudioAcoustics,
  pub(crate) reverb: Arc<ReverbParameters>,
}
