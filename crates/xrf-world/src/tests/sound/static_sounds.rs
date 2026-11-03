//! A level's static sounds, `SStaticSound::Update`: their windows, plays and pauses on the level's clock.

use std::time::Duration;

use xrf_engine_target::XrayEngine;

use crate::host::sound::world_sound_emitter::WorldSoundEmitter;
use crate::host::sound::world_static_sound::WorldStaticSound;
use crate::sound::sound_emitters::SoundEmitters;
use crate::sound::static_sounds::StaticSounds;
use crate::tests::sound::{found, missing, seconds};

fn record(active: [u32; 2], play: [u32; 2], pause: [u32; 2]) -> WorldStaticSound {
  WorldStaticSound {
    sound: found("ambient\\water", 4),
    position: [1.0, 2.0, 3.0],
    volume: 0.5,
    frequency: 1.0,
    active,
    play,
    pause,
  }
}

fn playing(emitters: &SoundEmitters, now: Duration) -> Vec<WorldSoundEmitter> {
  emitters.iter_playing(now).cloned().collect()
}

#[test]
fn loops_one_with_no_pause_for_ever_as_one_play() {
  let mut sounds: StaticSounds = StaticSounds::new(vec![record([0, 0], [0, 0], [0, 0])]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  sounds.update(&mut emitters, XrayEngine::Vanilla, 0, Duration::ZERO);

  let first: Vec<WorldSoundEmitter> = playing(&emitters, Duration::ZERO);

  assert_eq!(first.len(), 1);
  assert!(first[0].is_looped);
  assert_eq!(first[0].position, [1.0, 2.0, 3.0]);
  assert_eq!(first[0].volume, 0.5);

  for second in 1..100 {
    sounds.update(&mut emitters, XrayEngine::Vanilla, 0, seconds(second));
  }

  assert_eq!(
    playing(&emitters, seconds(99)).first().map(|it| it.id),
    Some(first[0].id)
  );
}

// No play window plays the sound through once, then waits its length and a drawn pause from its start.
#[test]
fn plays_one_through_and_waits_a_drawn_pause_after() {
  let mut sounds: StaticSounds = StaticSounds::new(vec![record([0, 0], [0, 0], [2000, 3000])]);
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut starts: Vec<Duration> = Vec::new();

  for tick in 0..2000 {
    let now: Duration = Duration::from_millis(tick * 10);

    sounds.update(&mut emitters, XrayEngine::Vanilla, 0, now);
    emitters.retire(now);

    for emitter in emitters.iter_playing(now) {
      assert!(!emitter.is_looped);

      if !starts.contains(&emitter.start) {
        starts.push(emitter.start);
      }
    }
  }

  assert!(starts.len() >= 2, "{starts:?}");

  for pair in starts.windows(2) {
    let gap: Duration = pair[1] - pair[0];

    assert!(gap >= seconds(6) && gap <= Duration::from_millis(7010), "{gap:?}");
  }
}

// A play window loops the sound until a drawn stop, which fades it out deferred, and waits a drawn pause after it.
#[test]
fn loops_one_until_its_drawn_stop_and_fades_it_out() {
  let mut sounds: StaticSounds = StaticSounds::new(vec![record([0, 0], [1000, 1000], [5000, 5000])]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  sounds.update(&mut emitters, XrayEngine::Vanilla, 0, Duration::ZERO);

  let id: u64 = playing(&emitters, Duration::ZERO)[0].id;

  assert!(playing(&emitters, Duration::ZERO)[0].is_looped);

  sounds.update(&mut emitters, XrayEngine::Vanilla, 0, seconds(1));

  assert_eq!(playing(&emitters, seconds(1))[0].stop, Some(seconds(1)));
  assert!(emitters.is_playing(id, Duration::from_millis(1099)));
  assert!(!emitters.is_playing(id, Duration::from_millis(1100)));

  // Not played again until the pause after the stop has passed.
  sounds.update(&mut emitters, XrayEngine::Vanilla, 0, seconds(5));
  emitters.retire(seconds(5));

  assert!(playing(&emitters, seconds(5)).is_empty());

  sounds.update(&mut emitters, XrayEngine::Vanilla, 0, seconds(6));

  assert_eq!(playing(&emitters, seconds(6)).len(), 1);
}

// Outside its window of the game's day a sound starts none, and one playing fades out.
#[test]
fn follows_its_active_window_of_the_day() {
  let mut sounds: StaticSounds = StaticSounds::new(vec![record([1000, 2000], [0, 0], [0, 0])]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  sounds.update(&mut emitters, XrayEngine::Vanilla, 500, Duration::ZERO);

  assert!(playing(&emitters, Duration::ZERO).is_empty());

  sounds.update(&mut emitters, XrayEngine::Vanilla, 1500, seconds(1));

  assert_eq!(playing(&emitters, seconds(1)).len(), 1);

  sounds.update(&mut emitters, XrayEngine::Vanilla, 2000, seconds(2));

  assert_eq!(playing(&emitters, seconds(2))[0].stop, Some(seconds(2)));
}

#[test]
fn plays_nothing_for_a_missing_file() {
  let mut sounds: StaticSounds = StaticSounds::new(vec![WorldStaticSound {
    sound: missing("ambient\\gone"),
    ..record([0, 0], [0, 0], [0, 0])
  }]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  sounds.update(&mut emitters, XrayEngine::Vanilla, 0, Duration::ZERO);

  assert!(playing(&emitters, Duration::ZERO).is_empty());
}
