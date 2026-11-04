use std::sync::Arc;

use rtrb::RingBuffer;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::control::audio_engine::AudioEngine;
use crate::control::audio_engine_settings::AudioEngineSettings;
use crate::control::engine_shared::EngineShared;
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
  /// An engine and the mixer it drives, sized by `settings`, playing dry.
  pub fn new(settings: AudioEngineSettings) -> Self {
    Self::with_acoustics(settings, AudioAcoustics::default())
  }

  /// An engine and the mixer it drives, sized by `settings`, its placed voices heard through `acoustics`.
  pub fn with_acoustics(settings: AudioEngineSettings, acoustics: AudioAcoustics) -> Self {
    let voices_count: usize = settings.voices_count.max(1);
    let (commands, mixer_commands) = RingBuffer::new(settings.commands_count.max(1));
    let (mixer_retired, retired) = RingBuffer::new(voices_count);
    let shared: EngineShared = EngineShared {
      parameters: (0..voices_count).map(|_| VoiceParameters::default()).collect(),
      statuses: (0..voices_count).map(|_| VoiceStatus::default()).collect(),
      meter: AudioMixerMeter::default(),
      acoustics,
      reverb: Arc::default(),
    };

    Self {
      engine: AudioEngine::new(commands, retired, shared.clone()),
      mixer: AudioMixer::new(mixer_commands, mixer_retired, shared),
    }
  }
}
