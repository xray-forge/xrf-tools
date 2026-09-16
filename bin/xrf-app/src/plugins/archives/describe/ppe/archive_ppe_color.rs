use serde::Serialize;
use xrf_db::PpeColor;

use crate::plugins::archives::describe::animation::ArchiveAnimationChannel;

/// The three channels a colour parameter is assembled from, in the order the file stores them.
const CHANNELS: [&str; 3] = ["red", "green", "blue"];

/// One colour parameter of an effect, and the three channels it is assembled from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePpeColor {
  pub name: String,
  /// `m_fBase`, stored ahead of the envelopes and never read at runtime - `update` assembles the colour from the
  /// three channels alone.
  pub base: f32,
  /// Keys across all three channels.
  pub keys: usize,
  /// Seconds the longest of the three spans, which is what this parameter contributes to the effect's own length.
  pub length_seconds: f32,
  pub channels: Vec<ArchiveAnimationChannel>,
}

impl ArchivePpeColor {
  /// Every colour of an effect, each under the name its position gives it.
  pub fn of_all<'a>(names: impl IntoIterator<Item = &'a str>, colors: &[PpeColor]) -> Vec<Self> {
    names
      .into_iter()
      .zip(colors)
      .map(|(name, color)| Self::of(name, color))
      .collect()
  }

  /// One colour, taken over the three channels it holds.
  fn of(name: &str, color: &PpeColor) -> Self {
    Self {
      name: name.to_owned(),
      base: color.base,
      keys: color.get_keys_count(),
      length_seconds: color.get_length_seconds(),
      channels: CHANNELS
        .into_iter()
        .zip(color.channels())
        .map(|(channel, envelope)| ArchiveAnimationChannel::of(channel, envelope))
        .collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_db::{AnimationEnvelope, AnimationKey, PPE_COLORS, PpeColor};

  use super::ArchivePpeColor;

  fn envelope(times: &[f32]) -> AnimationEnvelope {
    AnimationEnvelope {
      behavior: (1, 1),
      keys: times
        .iter()
        .map(|time| AnimationKey {
          value: 1.0,
          time: *time,
          shape: 0,
          interpolation: None,
        })
        .collect(),
    }
  }

  fn color(base: f32) -> PpeColor {
    PpeColor {
      base,
      red: envelope(&[0.0, 2.0]),
      green: envelope(&[0.0, 5.0]),
      blue: envelope(&[]),
    }
  }

  #[test]
  fn a_colour_names_each_of_the_three_channels_the_engine_assembles_it_from() {
    let colors: Vec<ArchivePpeColor> = ArchivePpeColor::of_all(PPE_COLORS, &[color(0.5)]);

    assert_eq!(colors.len(), 1);
    assert_eq!(colors[0].name, "base color");
    assert_eq!(
      colors[0]
        .channels
        .iter()
        .map(|channel| channel.name.as_str())
        .collect::<Vec<_>>(),
      vec!["red", "green", "blue"]
    );
  }

  #[test]
  fn a_colour_is_as_long_as_its_longest_channel_and_counts_every_key() {
    // The engine's own `get_keys_count` answers with red alone, which would say 2 here rather than 4.
    let colors: Vec<ArchivePpeColor> = ArchivePpeColor::of_all(PPE_COLORS, &[color(0.5)]);

    assert_eq!(colors[0].length_seconds, 5.0);
    assert_eq!(colors[0].keys, 4);
  }

  #[test]
  fn the_unread_base_is_carried_rather_than_dropped() {
    // Stored by every effect and read by none of them, which is a thing about the file worth being able to see.
    let colors: Vec<ArchivePpeColor> = ArchivePpeColor::of_all(PPE_COLORS, &[color(0.25)]);

    assert_eq!(colors[0].base, 0.25);
  }
}
