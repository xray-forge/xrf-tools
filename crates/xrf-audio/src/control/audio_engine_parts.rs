use std::sync::Arc;

use rtrb::RingBuffer;

use crate::control::audio_engine::AudioEngine;
use crate::control::audio_engine_settings::AudioEngineSettings;
use crate::mixer::audio_mixer::AudioMixer;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_status::VoiceStatus;

/// A new engine's two halves: the control side kept by the caller, and the mixer handed to a device or an offline
/// render.
pub struct AudioEngineParts {
  pub engine: AudioEngine,
  pub mixer: AudioMixer,
}

impl AudioEngineParts {
  /// An engine and the mixer it drives, sized by `settings`.
  pub fn new(settings: AudioEngineSettings) -> Self {
    let voices_count: usize = settings.voices_count.max(1);
    let (commands, mixer_commands) = RingBuffer::new(settings.commands_count.max(1));
    let (mixer_retired, retired) = RingBuffer::new(voices_count);
    let parameters: Arc<[VoiceParameters]> = (0..voices_count).map(|_| VoiceParameters::default()).collect();
    let statuses: Arc<[VoiceStatus]> = (0..voices_count).map(|_| VoiceStatus::default()).collect();
    let meter: AudioMixerMeter = AudioMixerMeter::default();

    Self {
      engine: AudioEngine::new(commands, retired, parameters.clone(), statuses.clone(), meter.clone()),
      mixer: AudioMixer::new(mixer_commands, mixer_retired, parameters, statuses, meter),
    }
  }
}
