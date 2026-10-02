use std::sync::Arc;

use xrf_engine_target::XrayEngine;

use super::mixer::{read_cycle, vanilla_fixture};
use crate::{
  WeatherDescriptor, WeatherEffectStart, WeatherEffectTimeline, WeatherMixer, WeatherPair, WeatherPlayedKeyframe,
};

/// Midnight, six, noon and nine at night.
fn cycle() -> Vec<WeatherPlayedKeyframe> {
  read_cycle(&vanilla_fixture(), XrayEngine::Vanilla)
    .0
    .into_iter()
    .map(|keyframe| WeatherPlayedKeyframe::of(Arc::new(keyframe)))
    .collect()
}

/// The same day with every keyframe an hour later and its fog a metre off, as another cycle.
fn other() -> Vec<WeatherPlayedKeyframe> {
  cycle()
    .into_iter()
    .map(|keyframe| {
      let descriptor: WeatherDescriptor = WeatherDescriptor {
        fog_distance: 1.0,
        time: keyframe.descriptor.time + 3_600,
        ..WeatherDescriptor::clone(&keyframe.descriptor)
      };

      WeatherPlayedKeyframe::of(Arc::new(descriptor))
    })
    .collect()
}

fn to_times(pair: &WeatherPair) -> Vec<f32> {
  pair
    .get()
    .map_or_else(Vec::new, |pair| pair.iter().map(|it| it.time).collect())
}

#[test]
fn selects_the_pair_around_the_time_on_a_forced_start_and_moves_on_only_past_the_second() {
  let cycle: Vec<WeatherPlayedKeyframe> = cycle();
  let mut pair: WeatherPair = WeatherPair::default();

  pair.advance(&cycle, 30_000.0);
  assert_eq!(to_times(&pair), [21_600.0, 43_200.0]);

  pair.advance(&cycle, 43_200.0);
  assert_eq!(to_times(&pair), [21_600.0, 43_200.0]);

  pair.advance(&cycle, 43_201.0);
  assert_eq!(to_times(&pair), [43_200.0, 75_600.0]);
}

// `SetWeather(name, false)`: the pair being blended stays, and the next keyframe comes from the weather playing then.
#[test]
fn keeps_blending_its_pair_when_the_weather_changes_under_it_taking_the_next_from_the_new_one() {
  let (cycle, other) = (cycle(), other());
  let mut pair: WeatherPair = WeatherPair::default();

  pair.advance(&cycle, 30_000.0);
  pair.advance(&other, 40_000.0);

  assert_eq!(to_times(&pair), [21_600.0, 43_200.0]);
  assert_eq!(
    pair.get().unwrap()[1].descriptor.fog_distance,
    cycle[2].descriptor.fog_distance
  );

  pair.advance(&other, 43_300.0);

  assert_eq!(to_times(&pair), [43_200.0, 46_800.0]);
  assert_eq!(pair.get().unwrap()[1].descriptor.fog_distance, 1.0);
}

#[test]
fn moves_on_across_midnight_only_once_the_time_is_between_the_two() {
  let cycle: Vec<WeatherPlayedKeyframe> = cycle();
  let mut pair: WeatherPair = WeatherPair::default();

  pair.advance(&cycle, 80_000.0);
  assert_eq!(to_times(&pair), [75_600.0, 0.0]);

  pair.advance(&cycle, 86_000.0);
  assert_eq!(to_times(&pair), [75_600.0, 0.0]);

  pair.advance(&cycle, 100.0);
  assert_eq!(to_times(&pair), [0.0, 21_600.0]);
}

#[test]
fn stays_on_the_one_keyframe_of_a_cycle_of_one() {
  let manual: Vec<WeatherPlayedKeyframe> = vec![cycle()[2].clone()];
  let mut pair: WeatherPair = WeatherPair::default();

  pair.advance(&manual, 1_000.0);
  pair.advance(&manual, 60_000.0);

  assert_eq!(pair.get(), Some(&[manual[0].clone(), manual[0].clone()]));
}

/// An effect a minute a keyframe, its own first replaced by the cycle's next as the engine replaces it.
fn effect(cycle: &[WeatherPlayedKeyframe]) -> Vec<WeatherPlayedKeyframe> {
  [0u32, 60, 120]
    .into_iter()
    .map(|time| {
      WeatherPlayedKeyframe::of(Arc::new(WeatherDescriptor {
        fog_distance: 10.0 + time as f32,
        time,
        ..WeatherDescriptor::clone(&cycle[0].descriptor)
      }))
    })
    .collect()
}

fn timeline(cycle: &[WeatherPlayedKeyframe], time: f32, factor: f32) -> Option<WeatherEffectTimeline> {
  let mut pair: WeatherPair = WeatherPair::default();

  pair.force(cycle, time);

  WeatherEffectTimeline::new(WeatherEffectStart {
    name: "fx",
    effect: &effect(cycle),
    cycle,
    current: pair.get().unwrap(),
    time,
    factor,
  })
}

#[test]
fn leads_an_effect_in_at_the_clock_weight_and_hands_the_cycle_back_its_next_keyframes() {
  let cycle: Vec<WeatherPlayedKeyframe> = cycle();
  // Seven in the morning, a sixth of the way from six to noon, at twelve game seconds a real one.
  let timeline: WeatherEffectTimeline = timeline(&cycle, 25_200.0, 12.0).unwrap();
  let times: Vec<f32> = timeline.keyframes.iter().map(|it| it.time).collect();

  assert_eq!(times, [25_188.0, 25_260.0, 25_320.0, 25_380.0, 25_440.0]);
  assert_eq!([timeline.start[0].time, timeline.start[1].time], [25_188.0, 25_260.0]);
  assert!((WeatherMixer::weigh(25_200.0, [25_188.0, 25_260.0]) - 1.0 / 6.0).abs() < 1e-6);
  // The lead-in reaches the cycle's noon, the effect's own first keyframe standing in for it.
  assert_eq!(
    timeline.keyframes[1].descriptor.fog_distance,
    cycle[2].descriptor.fog_distance
  );
  assert_eq!(timeline.keyframes[2].descriptor.fog_distance, 70.0);
  // `WFX_end_desc`: the cycle's noon, at or after the effect's last, and the keyframe after it; the lead-out copies
  // the first a lead-in past the effect's last, which is when the effect ends.
  assert_eq!(timeline.end, [cycle[2].clone(), cycle[3].clone()]);
  assert_eq!(
    timeline.keyframes[4].descriptor.fog_distance,
    cycle[2].descriptor.fog_distance
  );
  assert_eq!(timeline.duration, 25_440.0 - 25_200.0);
}

#[test]
fn carries_an_effect_weight_across_midnight() {
  let cycle: Vec<WeatherPlayedKeyframe> = cycle();
  let timeline: WeatherEffectTimeline = timeline(&cycle, 86_300.0, 100.0).unwrap();
  let keyframes: &[WeatherPlayedKeyframe] = &timeline.keyframes;

  assert!(keyframes.iter().all(|it| (0.0..86_400.0).contains(&it.time)));
  // The lead-in sorts last and the cycle's midnight first, and the weight between them is the one the clock was at.
  let weight: f32 = WeatherMixer::weigh(86_300.0, [keyframes[keyframes.len() - 1].time, keyframes[0].time]);

  assert!((weight - (86_300.0 - 75_600.0) / 10_800.0).abs() < 1e-4);
}

#[test]
fn plays_no_effect_without_keyframes() {
  let cycle: Vec<WeatherPlayedKeyframe> = cycle();
  let mut pair: WeatherPair = WeatherPair::default();

  pair.force(&cycle, 0.0);

  assert_eq!(
    WeatherEffectTimeline::new(WeatherEffectStart {
      name: "fx",
      effect: &[],
      cycle: &cycle,
      current: pair.get().unwrap(),
      time: 0.0,
      factor: 12.0,
    }),
    None
  );
}
