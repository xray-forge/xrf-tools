use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::ambient::sound_channel::SoundChannel;
use crate::finding::EnvironmentRule;
use crate::key::{EnvironmentValue, declare_environment_keys};
use crate::section::EnvironmentSectionReader;

declare_environment_keys! {
  /// The keys of a `sound_channels.ltx` section, `CEnvAmbient::SSndChannel::load`: one set of sounds an ambient plays
  /// around the camera, at random within a distance and a period.
  pub enum SoundChannelKey {
    /// Metres, nearest then farthest, in place of the two keys; either key still overrides its half.
    SoundDist = "sound_dist": Vector { least: 2, most: 2 }, vanilla: Conditional(Vector(&[0.0, 0.0])), extended: Unread;
    /// Metres.
    MinDistance = "min_distance": Number, vanilla: Conditional(Number(0.0)), extended: Required(Number(0.0));
    /// Metres.
    MaxDistance = "max_distance": Number, vanilla: Conditional(Number(0.0)), extended: Required(Number(0.0));
    /// Seconds: four as pre-Clear Sky writes them, or Shadow of Chernobyl's two for both halves.
    SoundPeriod = "sound_period": Vector { least: 2, most: 4 },
      vanilla: Conditional(Vector(&[0.0, 0.0, 0.0, 0.0])), extended: Unread;
    /// Milliseconds before a sound first plays, least of a range.
    Period0 = "period0": Integer, vanilla: Conditional(Integer(0)), extended: Required(Integer(0));
    /// Milliseconds, most of the same range.
    Period1 = "period1": Integer, vanilla: Conditional(Integer(0)), extended: Required(Integer(0));
    /// Milliseconds between sounds after, least of a range.
    Period2 = "period2": Integer, vanilla: Conditional(Integer(0)), extended: Required(Integer(0));
    /// Milliseconds, most of the same range.
    Period3 = "period3": Integer, vanilla: Conditional(Integer(0)), extended: Required(Integer(0));
    /// Sounds under `sounds`, one of which plays at a time; the engine asserts on none.
    Sounds = "sounds": List, vanilla: Required(List), extended: Required(List);
  }
}

impl SoundChannelKey {
  const PERIODS: [Self; 4] = [Self::Period0, Self::Period1, Self::Period2, Self::Period3];

  /// Reads one sound channel and says what its engine would refuse.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> SoundChannel {
    let channel: SoundChannel = reader.read::<Self>(name, section);

    Self::judge(reader, &channel);

    channel
  }

  /// Checks a channel read, which for a Shadow of Chernobyl ambient is the ambient's own section.
  pub(crate) fn judge(reader: &mut EnvironmentSectionReader, channel: &SoundChannel) {
    let engine: XrayEngine = reader.get_engine();
    let name: &str = &channel.name;
    let is_distance_pair: bool = engine == XrayEngine::Extended || !channel.has(Self::SoundDist);

    if is_distance_pair {
      if engine == XrayEngine::Vanilla {
        reader.require(channel, Self::MinDistance);
        reader.require(channel, Self::MaxDistance);
      }

      // `R_ASSERT2(m_sound_dist.y > m_sound_dist.x, sect)`, asked only where the two keys are the distances.
      let (near, far) = (
        channel.get_number(Self::MinDistance, engine),
        channel.get_number(Self::MaxDistance, engine),
      );

      if channel.has(Self::MinDistance) && channel.has(Self::MaxDistance) && far <= near {
        let message: String = format!(
          "{} has [max_distance] {far} not beyond [min_distance] {near}, which the engine asserts on",
          reader.describe(name)
        );

        reader.report(EnvironmentRule::Engine, name, Some("max_distance"), message);
      }
    }

    if engine == XrayEngine::Vanilla && !channel.has(Self::SoundPeriod) {
      for key in Self::PERIODS {
        reader.require(channel, key);
      }
    }

    // `R_ASSERT(m_sound_period.x <= m_sound_period.y && m_sound_period.z <= m_sound_period.w)`.
    let [first, second, third, fourth] = Self::read_periods(channel, engine);

    if first > second || third > fourth {
      let message: String = format!(
        "{} has a period whose least exceeds its most ({first}..{second}, {third}..{fourth}), which the engine asserts on",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Engine, name, None, message);
    }

    if channel.has(Self::Sounds) && channel.get_list(Self::Sounds).is_empty() {
      let message: String = format!("{} has no [sounds], which the engine asserts on", reader.describe(name));

      reader.report(EnvironmentRule::Engine, name, Some("sounds"), message);
    }
  }

  /// The four periods as the engine compares them: each key where written, else `sound_period`'s component, two of
  /// which stand for all four in Shadow of Chernobyl's form. Units differ between the forms; the order does not.
  fn read_periods(channel: &SoundChannel, engine: XrayEngine) -> [f64; 4] {
    let written: &[f32] = channel
      .get(Self::SoundPeriod)
      .and_then(EnvironmentValue::as_vector)
      .unwrap_or(&[]);
    let fallback: [f32; 4] = match written {
      [least, most] => [*least, *most, *least, *most],
      _ => std::array::from_fn(|index| written.get(index).copied().unwrap_or(0.0)),
    };

    std::array::from_fn(|index| {
      let key: Self = Self::PERIODS[index];

      if channel.has(key) || engine == XrayEngine::Extended {
        f64::from(channel.get_integer(key, engine))
      } else {
        f64::from(fallback[index])
      }
    })
  }
}
