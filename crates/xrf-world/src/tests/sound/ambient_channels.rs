//! The weather ambient's sound channels, `CGamePersistent::WeathersUpdate`: first waits, places and later waits.

use std::collections::HashMap;
use std::time::Duration;

use glam::Vec3;

use crate::host::sound::world_sound_channel::WorldSoundChannel;
use crate::host::world_ambient::WorldAmbient;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::sound::ambient_channels::AmbientChannels;
use crate::sound::sound_emitters::SoundEmitters;
use crate::tests::sound::found;
use crate::weather::weather_ambients::WeatherAmbients;

const EYE: Vec3 = Vec3::new(100.0, 5.0, -40.0);

fn weather(period: [u32; 4]) -> WorldLevelWeather {
  WorldLevelWeather {
    ambients: HashMap::from([(
      String::from("day"),
      WorldAmbient {
        sound_channels: vec![String::from("birds")],
        ..WorldAmbient::default()
      },
    )]),
    sound_channels: HashMap::from([(
      String::from("birds"),
      WorldSoundChannel {
        sounds: vec![found("ambient\\bird_1", 2), found("ambient\\bird_2", 2)],
        distance: [20.0, 40.0],
        period,
      },
    )]),
    ..WorldLevelWeather::default()
  }
}

fn day() -> WeatherAmbients {
  WeatherAmbients {
    names: [Some(String::from("day")), Some(String::from("day"))],
    weight: 0.0,
  }
}

/// When each sound started and where, over `ticks` ticks of 10 ms.
fn play(level: &WorldLevelWeather, ticks: u64) -> Vec<(Duration, [f32; 3])> {
  let mut channels: AmbientChannels = AmbientChannels::default();
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut starts: Vec<(Duration, [f32; 3])> = Vec::new();

  for tick in 0..ticks {
    let now: Duration = Duration::from_millis(tick * 10);

    channels.update(&mut emitters, level, &day(), EYE, now);
    emitters.retire(now);

    for emitter in emitters.iter_playing(now) {
      if !starts.iter().any(|(start, _)| *start == emitter.start) {
        starts.push((emitter.start, emitter.position));
      }
    }
  }

  starts
}

#[test]
fn plays_around_the_camera_after_its_drawn_waits() {
  let starts: Vec<(Duration, [f32; 3])> = play(&weather([1000, 2000, 3000, 4000]), 2500);

  assert!(starts.len() >= 3, "{starts:?}");
  assert!(starts[0].0 > Duration::from_millis(1000) && starts[0].0 <= Duration::from_millis(2010));

  for pair in starts.windows(2) {
    let gap: Duration = pair[1].0 - pair[0].0;

    // The sound's two seconds, then three to four.
    assert!(
      gap > Duration::from_millis(5000) && gap <= Duration::from_millis(6010),
      "{gap:?}"
    );
  }

  for (_, position) in &starts {
    let offset: Vec3 = Vec3::from(*position) - EYE;
    let level: f32 = Vec3::new(offset.x, 0.0, offset.z).length();

    assert_eq!(offset.y, 10.0);
    assert!((20.0..=40.0).contains(&level), "{level}");
  }
}

// `(x < y) ? randI(x, y) : 0`: an empty range waits nothing, not its least.
#[test]
fn waits_nothing_for_an_empty_range() {
  let starts: Vec<(Duration, [f32; 3])> = play(&weather([500, 500, 700, 700]), 600);

  assert_eq!(starts[0].0, Duration::from_millis(10));
  assert_eq!(starts[1].0 - starts[0].0, Duration::from_millis(2010));
}
