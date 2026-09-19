use serde::Serialize;
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey};

use crate::plugins::archives::describe::animation::archive_animation_behavior::ArchiveAnimationBehavior;
use crate::plugins::archives::describe::animation::archive_animation_shape::ArchiveAnimationShape;

/// One animated channel: what it drives, and the keys that drive it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAnimationChannel {
  /// What the channel animates, which is its position in the file rather than anything the file names.
  pub name: String,
  pub keys: usize,
  /// Seconds its first key sits at, absent for a channel carrying none.
  pub first_seconds: Option<f32>,
  /// Seconds its last key sits at, absent for a channel carrying none.
  pub last_seconds: Option<f32>,
  /// The smallest value any of its keys holds, absent for a channel carrying none.
  pub minimum: Option<f32>,
  /// The largest value any of its keys holds, absent for a channel carrying none.
  pub maximum: Option<f32>,
  /// What it does before its first key.
  pub behavior_before: ArchiveAnimationBehavior,
  /// What it does after its last key.
  pub behavior_after: ArchiveAnimationBehavior,
  /// Curve shapes its keys use, named, each once.
  pub shapes: Vec<String>,
}

impl ArchiveAnimationChannel {
  /// Every envelope of a fixed run, each under the name its position gives it.
  pub fn of_all<'a>(names: impl IntoIterator<Item = &'a str>, envelopes: &[AnimationEnvelope]) -> Vec<Self> {
    names
      .into_iter()
      .zip(envelopes)
      .map(|(name, envelope)| Self::of(name, envelope))
      .collect()
  }

  /// One channel, taken over the keys it carries.
  pub fn of(name: &str, envelope: &AnimationEnvelope) -> Self {
    let (before, after): (u8, u8) = envelope.behavior;

    Self {
      name: name.to_owned(),
      keys: envelope.keys.len(),
      first_seconds: envelope.keys.first().map(|key| key.time),
      last_seconds: envelope.keys.last().map(|key| key.time),
      minimum: fold_values(envelope, f32::min),
      maximum: fold_values(envelope, f32::max),
      behavior_before: ArchiveAnimationBehavior::of(before),
      behavior_after: ArchiveAnimationBehavior::of(after),
      shapes: ArchiveAnimationShape::label_all(envelope.keys.iter().map(|key| key.shape)),
    }
  }

  /// Seconds between its first and last key, which is `CEnvelope::GetLength`.
  pub fn get_length_seconds(&self) -> f32 {
    match (self.first_seconds, self.last_seconds) {
      (Some(first), Some(last)) => last - first,
      _ => 0.0,
    }
  }
}

/// The one value a fold over a channel's keys leaves, or `None` for a channel carrying none.
fn fold_values(envelope: &AnimationEnvelope, fold: fn(f32, f32) -> f32) -> Option<f32> {
  envelope
    .keys
    .iter()
    .map(|key: &AnimationKey| key.value)
    .reduce(fold)
    .filter(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
  use xrf_animation_envelope::{AnimationEnvelope, AnimationKey};
  use xrf_anm::ANM_CHANNELS;

  use super::ArchiveAnimationChannel;
  use crate::plugins::archives::describe::animation::archive_animation_behavior::ArchiveAnimationBehavior;

  fn key(value: f32, time: f32, shape: u8) -> AnimationKey {
    AnimationKey {
      value,
      time,
      shape,
      interpolation: None,
    }
  }

  fn envelope(behavior: (u8, u8), keys: Vec<AnimationKey>) -> AnimationEnvelope {
    AnimationEnvelope { behavior, keys }
  }

  #[test]
  fn a_channel_is_named_by_the_position_it_holds() {
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      ANM_CHANNELS,
      &[envelope((1, 1), vec![key(0.0, 0.0, 0)]), envelope((1, 1), Vec::new())],
    );

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].name, "position x");
    assert_eq!(channels[1].name, "position y");
  }

  #[test]
  fn a_name_no_envelope_answers_describes_nothing() {
    // A format naming more channels than the file carries describes the ones it has, not an empty row for the rest.
    let channels: Vec<ArchiveAnimationChannel> =
      ArchiveAnimationChannel::of_all(["red", "green", "blue"], &[envelope((1, 1), Vec::new())]);

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].name, "red");
  }

  #[test]
  fn a_channel_reports_the_span_and_the_reach_of_its_own_keys() {
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      ANM_CHANNELS,
      &[envelope(
        (1, 1),
        vec![key(0.5, 0.0, 0), key(-1.25, 1.5, 3), key(2.0, 4.0, 0)],
      )],
    );

    assert_eq!(channels[0].keys, 3);
    assert_eq!(
      (channels[0].first_seconds, channels[0].last_seconds),
      (Some(0.0), Some(4.0))
    );
    assert_eq!((channels[0].minimum, channels[0].maximum), (Some(-1.25), Some(2.0)));
    assert_eq!(channels[0].shapes, vec![String::from("tcb"), String::from("linear")]);
    assert_eq!(channels[0].get_length_seconds(), 4.0);
  }

  #[test]
  fn a_channel_carrying_no_keys_reports_no_span_rather_than_an_empty_one() {
    let channels: Vec<ArchiveAnimationChannel> =
      ArchiveAnimationChannel::of_all(ANM_CHANNELS, &[envelope((1, 1), Vec::new())]);

    assert_eq!(channels[0].keys, 0);
    assert_eq!(channels[0].first_seconds, None);
    assert_eq!(channels[0].minimum, None);
    assert!(channels[0].shapes.is_empty());
    assert_eq!(channels[0].get_length_seconds(), 0.0);
  }

  #[test]
  fn a_channel_says_what_it_does_at_either_end_of_its_keys() {
    let channels: Vec<ArchiveAnimationChannel> =
      ArchiveAnimationChannel::of_all(ANM_CHANNELS, &[envelope((1, 2), vec![key(0.0, 0.0, 0)])]);

    assert_eq!(channels[0].behavior_before, ArchiveAnimationBehavior::Constant);
    assert_eq!(channels[0].behavior_after, ArchiveAnimationBehavior::Repeat);
  }
}
