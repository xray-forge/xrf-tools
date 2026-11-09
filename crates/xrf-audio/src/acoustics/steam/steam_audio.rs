#![expect(
  unsafe_code,
  reason = "calls Steam Audio's air absorption model, which audionimbus exposes as unsafe"
)]

use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use audionimbus::{AirAbsorptionModel, AudioSettings, Context, ContextSettings, Hrtf, HrtfSettings};

use crate::acoustics::steam::mixer_reverb::MixerReverb;
use crate::acoustics::steam::steam_allocator::SteamAllocator;
use crate::acoustics::steam::steam_library::SteamLibrary;
use crate::acoustics::steam::steam_space::SteamSpace;
use crate::acoustics::steam::voice_effects::VoiceEffects;
use crate::mixer::audio_mixer::AudioMixer;

/// Steam Audio, loaded and started: its context, and the head-related transfer function at each rate a mixer asked
/// for, created once each. Effects are made here off the audio thread and moved onto it with their voice.
pub(crate) struct SteamAudio {
  context: Context,
  /// Each rate's transfer function, by the rate.
  hrtfs: Mutex<HashMap<u32, Hrtf>>,
}

impl SteamAudio {
  /// Loads the library from `directory` and starts it; answers why it cannot be, including a library of another
  /// version than this build binds, which the context refuses before anything else of it is called.
  pub(crate) fn load(directory: &Path) -> Result<Self, String> {
    SteamLibrary::load(directory)?;

    let settings: ContextSettings = ContextSettings::new()
      .with_allocate_callback(SteamAllocator::allocate)
      .with_free_callback(SteamAllocator::free);
    let context: Context =
      Context::try_new(&settings).map_err(|error| format!("Steam Audio does not start: {error}"))?;
    let steam: Self = Self {
      context,
      hrtfs: Mutex::new(HashMap::new()),
    };

    steam.get_hrtf(AudioMixer::DEFAULT_OUTPUT_RATE)?;

    Ok(steam)
  }

  pub(crate) fn get_context(&self) -> &Context {
    &self.context
  }

  /// What every effect processes: one mixer block at a time, at `rate`.
  pub(crate) fn get_audio_settings(rate: u32) -> AudioSettings {
    AudioSettings {
      sampling_rate: rate,
      frame_size: AudioMixer::BLOCK_FRAMES as u32,
    }
  }

  /// The default transfer function at `rate`, created the first time a rate is asked for.
  pub(crate) fn get_hrtf(&self, rate: u32) -> Result<Hrtf, String> {
    let mut hrtfs = self.hrtfs.lock().unwrap_or_else(PoisonError::into_inner);

    if let Some(hrtf) = hrtfs.get(&rate) {
      return Ok(hrtf.clone());
    }

    let hrtf: Hrtf = Hrtf::try_new(&self.context, &Self::get_audio_settings(rate), &HrtfSettings::default())
      .map_err(|error| format!("Steam Audio's HRTF does not load at {rate} Hz: {error}"))?;

    hrtfs.insert(rate, hrtf.clone());

    Ok(hrtf)
  }

  /// A placed voice's effects at `rate`.
  pub(crate) fn create_voice_effects(&self, rate: u32) -> Result<VoiceEffects, String> {
    VoiceEffects::new(&self.context, &Self::get_audio_settings(rate), self.get_hrtf(rate)?)
  }

  /// The listener's reverb at `rate`.
  pub(crate) fn create_reverb(&self, rate: u32) -> Result<MixerReverb, String> {
    thread_local! {
      static IS_REVERB_MADE: Cell<bool> = const { Cell::new(false) };
    }

    // The first parametric reverb made on a thread renders the same input differently from every one made after it
    // there, so one is made and let go first, and every reverb renders alike.
    if !IS_REVERB_MADE.replace(true) {
      drop(MixerReverb::new(&self.context, &Self::get_audio_settings(rate)));
    }

    MixerReverb::new(&self.context, &Self::get_audio_settings(rate))
  }

  /// How much of each band the air lets through between `source` and `listener`, in the game's space, by its default
  /// model.
  pub(crate) fn get_air_absorption(&self, source: [f32; 3], listener: [f32; 3]) -> [f32; 3] {
    let (source, listener) = (SteamSpace::convert_point(source), SteamSpace::convert_point(listener));

    // SAFETY: the default model takes no callback, which is what the call cannot take safely.
    unsafe { audionimbus::model::air_absorption(&self.context, source, listener, &AirAbsorptionModel::Default) }.0
  }
}
