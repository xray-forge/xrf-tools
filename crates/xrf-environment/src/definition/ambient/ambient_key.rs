use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::definition::ambient::ambient::Ambient;
use crate::section::EnvironmentSectionReader;
use crate::section::finding::EnvironmentRule;
use crate::section::key::declare_environment_keys;

declare_environment_keys! {
  /// The keys of an `ambients.ltx` section, `CEnvAmbient::load` (`xrEngine/Environment_misc.cpp`): the sounds and
  /// particles a keyframe names by `ambient`.
  pub enum AmbientKey {
    /// Call of Pripyat's `sound_channels.ltx` sections, which OpenXRay prefers over either older spelling.
    SoundChannels = "sound_channels": List, vanilla: Conditional(List), extended: Required(List);
    /// Pre-Clear Sky's spelling of the same.
    SndChannels = "snd_channels": List, vanilla: Conditional(List), extended: Unread;
    /// Shadow of Chernobyl's: the ambient is its own sound channel, the `sound_channels.ltx` section of its name, which
    /// holds the sounds the engine plays; this list only marks it.
    Sounds = "sounds": List, vanilla: Conditional(List), extended: Unread;
    /// Seconds between effects, least then most, in place of the two keys.
    EffectPeriod = "effect_period": Vector { least: 2, most: 2 }, vanilla: Conditional(Vector(&[0.0, 0.0])), extended: Unread;
    /// Seconds.
    MinEffectPeriod = "min_effect_period": Number,
      vanilla: Conditional(Number(0.0)), extended: Required(Number(0.0));
    /// Seconds.
    MaxEffectPeriod = "max_effect_period": Number,
      vanilla: Conditional(Number(0.0)), extended: Required(Number(0.0));
    /// `effects.ltx` sections.
    Effects = "effects": List, vanilla: Optional(List), extended: Required(List);
  }
}

impl AmbientKey {
  /// Reads one ambient and says what its engine would refuse.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> Ambient {
    let engine: XrayEngine = reader.get_engine();
    let ambient: Ambient = reader.read::<Self>(name, section);

    if engine == XrayEngine::Vanilla && !ambient.has(Self::EffectPeriod) {
      reader.require(&ambient, Self::MinEffectPeriod);
      reader.require(&ambient, Self::MaxEffectPeriod);
    }

    // `R_ASSERT(!m_sound_channels.empty() || !m_effects.empty())`, which both engines ask.
    if Self::list_channels(&ambient, engine).is_empty() && ambient.get_list(Self::Effects).is_empty() {
      let message: String = format!("{} has neither sound channels nor effects", reader.describe(name));

      reader.report(EnvironmentRule::Engine, name, None, message);
    }

    ambient
  }

  /// Seconds between effects, least then most, `m_effect_period`: OpenXRay's `effect_period` where written, either of
  /// the two keys still reading over it, and the two keys alone otherwise.
  pub fn get_effect_period(ambient: &Ambient, engine: XrayEngine) -> [f32; 2] {
    let mut period: [f32; 2] = if engine == XrayEngine::Vanilla && ambient.has(Self::EffectPeriod) {
      ambient.get_vector::<2>(Self::EffectPeriod, engine)
    } else {
      [
        ambient.get_number(Self::MinEffectPeriod, engine),
        ambient.get_number(Self::MaxEffectPeriod, engine),
      ]
    };

    if engine == XrayEngine::Vanilla && ambient.has(Self::EffectPeriod) {
      for (slot, key) in [(0, Self::MinEffectPeriod), (1, Self::MaxEffectPeriod)] {
        if ambient.has(key) {
          period[slot] = ambient.get_number(key, engine);
        }
      }
    }

    period
  }

  /// The `sound_channels.ltx` sections the ambient plays: its list under whichever spelling its engine reads, or for
  /// an ambient that is its own channel, the section of its own name.
  pub fn list_channels(ambient: &Ambient, engine: XrayEngine) -> Vec<&str> {
    if Self::is_own_channel(ambient, engine) {
      return vec![ambient.name.as_str()];
    }

    let channels: &[String] = match engine {
      XrayEngine::Vanilla if !ambient.has(Self::SoundChannels) => ambient.get_list(Self::SndChannels),
      _ => ambient.get_list(Self::SoundChannels),
    };

    channels.iter().map(String::as_str).collect()
  }

  /// Whether the ambient is its own sound channel, as a Shadow of Chernobyl one is: `sounds` with neither channel
  /// list. Only OpenXRay reads one, and `CEnvAmbient::load` reads that channel from the sound channels config under the
  /// ambient's name, not from the ambient's own section.
  pub fn is_own_channel(ambient: &Ambient, engine: XrayEngine) -> bool {
    engine == XrayEngine::Vanilla
      && ambient.has(Self::Sounds)
      && !ambient.has(Self::SoundChannels)
      && !ambient.has(Self::SndChannels)
  }
}
