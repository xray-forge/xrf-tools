use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::ambient::ambient::Ambient;
use crate::finding::EnvironmentRule;
use crate::key::declare_environment_keys;
use crate::section::EnvironmentSectionReader;

declare_environment_keys! {
  /// The keys of an `ambients.ltx` section, `CEnvAmbient::load` (`xrEngine/Environment_misc.cpp`): the sounds and
  /// particles a keyframe names by `ambient`.
  pub enum AmbientKey {
    /// Call of Pripyat's `sound_channels.ltx` sections, which OpenXRay prefers over either older spelling.
    SoundChannels = "sound_channels": List, vanilla: Conditional(List), extended: Required(List);
    /// Pre-Clear Sky's spelling of the same.
    SndChannels = "snd_channels": List, vanilla: Conditional(List), extended: Unread;
    /// Shadow of Chernobyl's: the ambient is its own sound channel, and these are its sounds.
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
    let ambient: Ambient = reader.read::<Self>(name, section);

    if reader.get_engine() == XrayEngine::Vanilla {
      if !ambient.has(Self::EffectPeriod) {
        reader.require(&ambient, Self::MinEffectPeriod);
        reader.require(&ambient, Self::MaxEffectPeriod);
      }

      // `R_ASSERT(!m_sound_channels.empty() || !m_effects.empty())`; a Shadow of Chernobyl ambient is one channel.
      if Self::list_channels(&ambient).is_empty()
        && !ambient.has(Self::Sounds)
        && ambient.get_list(Self::Effects).is_empty()
      {
        let message: String = format!("{} has neither sound channels nor effects", reader.describe(name));

        reader.report(EnvironmentRule::Engine, name, None, message);
      }
    }

    ambient
  }

  /// The `sound_channels.ltx` sections the ambient plays, under whichever spelling its engine reads.
  pub fn list_channels(ambient: &Ambient) -> &[String] {
    if ambient.has(Self::SoundChannels) {
      ambient.get_list(Self::SoundChannels)
    } else {
      ambient.get_list(Self::SndChannels)
    }
  }

  /// Whether the ambient is its own sound channel, as a Shadow of Chernobyl one is: `sounds` with neither channel
  /// list. Only OpenXRay reads one.
  pub fn is_own_channel(ambient: &Ambient, engine: XrayEngine) -> bool {
    engine == XrayEngine::Vanilla
      && ambient.has(Self::Sounds)
      && !ambient.has(Self::SoundChannels)
      && !ambient.has(Self::SndChannels)
  }
}
