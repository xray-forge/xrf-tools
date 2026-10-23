use serde::Serialize;
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationShape};

use crate::plugins::archives::describe::animation::archive_animation_behavior::ArchiveAnimationBehavior;

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
    Self {
      name: name.to_owned(),
      keys: envelope.keys.len(),
      first_seconds: envelope.keys.first().map(|key| key.time),
      last_seconds: envelope.keys.last().map(|key| key.time),
      minimum: Self::fold_values(envelope, f32::min),
      maximum: Self::fold_values(envelope, f32::max),
      behavior_before: ArchiveAnimationBehavior::of(envelope.behavior_before),
      behavior_after: ArchiveAnimationBehavior::of(envelope.behavior_after),
      shapes: Self::label_shapes(envelope),
    }
  }

  /// Seconds between its first and last key, which is `CEnvelope::GetLength`.
  pub fn get_length_seconds(&self) -> f32 {
    match (self.first_seconds, self.last_seconds) {
      (Some(first), Some(last)) => last - first,
      _ => 0.0,
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

  /// The shapes a channel's keys use, each once, in the order the engine numbers them and in the terms an author
  /// would recognise. A shape the engine does not name keeps its number, so a file written by another tool does not
  /// read as using no shape at all.
  fn label_shapes(envelope: &AnimationEnvelope) -> Vec<String> {
    let mut used: Vec<u8> = envelope.keys.iter().map(|key| u8::from(key.shape)).collect();

    used.sort_unstable();
    used.dedup();

    used
      .into_iter()
      .map(|shape| match AnimationShape::from(shape) {
        AnimationShape::Tcb => String::from("tcb"),
        AnimationShape::Hermite => String::from("hermite"),
        AnimationShape::Bezier => String::from("bezier"),
        AnimationShape::Linear => String::from("linear"),
        AnimationShape::Stepped => String::from("stepped"),
        AnimationShape::TimedBezier => String::from("bezier 2"),
        AnimationShape::Unknown(shape) => format!("shape {shape}"),
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use xrf_animation_envelope::{AnimationBehavior, AnimationEnvelope, AnimationKey, AnimationShape};
  use xrf_anm::AnmChannel;

  use super::ArchiveAnimationChannel;
  use crate::plugins::archives::describe::animation::archive_animation_behavior::ArchiveAnimationBehavior;

  fn key(value: f32, time: f32, shape: AnimationShape) -> AnimationKey {
    AnimationKey {
      value,
      time,
      shape,
      ..AnimationKey::default()
    }
  }

  /// An object motion's channel names, the format names the tests take channels under.
  fn motion_channel_names() -> [&'static str; 6] {
    AnmChannel::ALL.map(AnmChannel::get_name)
  }

  fn envelope(keys: Vec<AnimationKey>) -> AnimationEnvelope {
    AnimationEnvelope {
      keys,
      ..AnimationEnvelope::default()
    }
  }

  #[test]
  fn a_channel_is_named_by_the_position_it_holds() {
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      motion_channel_names(),
      &[envelope(vec![key(0.0, 0.0, AnimationShape::Tcb)]), envelope(Vec::new())],
    );

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].name, "position x");
    assert_eq!(channels[1].name, "position y");
  }

  #[test]
  fn a_name_no_envelope_answers_describes_nothing() {
    // A format naming more channels than the file carries describes the ones it has, not an empty row for the rest.
    let channels: Vec<ArchiveAnimationChannel> =
      ArchiveAnimationChannel::of_all(["red", "green", "blue"], &[envelope(Vec::new())]);

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].name, "red");
  }

  #[test]
  fn a_channel_reports_the_span_and_the_reach_of_its_own_keys() {
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      motion_channel_names(),
      &[envelope(vec![
        key(0.5, 0.0, AnimationShape::Tcb),
        key(-1.25, 1.5, AnimationShape::Linear),
        key(2.0, 4.0, AnimationShape::Tcb),
      ])],
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
      ArchiveAnimationChannel::of_all(motion_channel_names(), &[envelope(Vec::new())]);

    assert_eq!(channels[0].keys, 0);
    assert_eq!(channels[0].first_seconds, None);
    assert_eq!(channels[0].minimum, None);
    assert!(channels[0].shapes.is_empty());
    assert_eq!(channels[0].get_length_seconds(), 0.0);
  }

  #[test]
  fn a_channel_says_what_it_does_at_either_end_of_its_keys() {
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      motion_channel_names(),
      &[AnimationEnvelope {
        behavior_after: AnimationBehavior::Repeat,
        ..envelope(vec![key(0.0, 0.0, AnimationShape::Tcb)])
      }],
    );

    assert_eq!(channels[0].behavior_before, ArchiveAnimationBehavior::Constant);
    assert_eq!(channels[0].behavior_after, ArchiveAnimationBehavior::Repeat);
  }

  #[test]
  fn a_channel_names_each_shape_its_keys_use_once_in_the_engine_s_order() {
    // The mix of a vanilla camera effect: mostly tcb with a few linear segments.
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      motion_channel_names(),
      &[envelope(
        [3, 0, 0, 5, 3]
          .into_iter()
          .map(|shape| key(0.0, 0.0, AnimationShape::from(shape)))
          .collect(),
      )],
    );

    assert_eq!(channels[0].shapes, vec!["tcb", "linear", "bezier 2"]);
  }

  #[test]
  fn a_shape_the_engine_does_not_name_keeps_its_number() {
    let channels: Vec<ArchiveAnimationChannel> = ArchiveAnimationChannel::of_all(
      motion_channel_names(),
      &[envelope(vec![key(0.0, 0.0, AnimationShape::from(7))])],
    );

    assert_eq!(channels[0].shapes, vec!["shape 7"]);
  }
}
