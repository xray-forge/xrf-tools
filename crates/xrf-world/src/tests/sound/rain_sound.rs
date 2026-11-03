//! The rain's loop, `CEffect_Rain::OnFrame`: when it starts and stops, and how loud, per engine.

use std::time::Duration;

use xrf_engine_target::XrayEngine;

use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_emitter::WorldSoundEmitter;
use crate::sound::rain_sound::{RainFrame, RainSound};
use crate::sound::sound_emitters::SoundEmitters;
use crate::tests::sound::found;

/// The loop after a second of frames at `frame`, its hemi eased all the way.
fn rain(engine: XrayEngine, frame: RainFrame) -> Option<WorldSoundEmitter> {
  let sound: WorldSound = found("ambient\\rain", 30);
  let mut rain: RainSound = RainSound::default();
  let mut emitters: SoundEmitters = SoundEmitters::default();

  for tick in 1..=200 {
    rain.update(
      &mut emitters,
      Some(&sound),
      engine,
      frame,
      Duration::from_millis(50),
      Duration::from_millis(tick * 50),
    );
  }

  emitters.iter_playing(Duration::from_secs(10)).next().cloned()
}

#[test]
fn loops_at_the_origin_once_it_rains_and_stops_when_it_does_not() {
  let wet: RainFrame = RainFrame {
    density: 0.4,
    wind_velocity: 0.0,
    hemi: Some(1.0),
  };
  let playing: WorldSoundEmitter = rain(XrayEngine::Vanilla, wet).expect("it rains");

  assert!(playing.is_looped);
  assert_eq!(playing.position, [0.0; 3]);
  assert_eq!([playing.min_distance, playing.max_distance], [40.0, 80.0]);
  assert!(rain(XrayEngine::Vanilla, RainFrame { density: 0.0, ..wet }).is_none());
}

// OpenXRay: `max(0.1, density) * hemi`. Monolith: half density and half wind, `clamp(.., 0.1, 1)`, its range halved.
#[test]
fn is_as_loud_as_each_engine_makes_it() {
  let frame: RainFrame = RainFrame {
    density: 0.05,
    wind_velocity: 600.0,
    hemi: Some(0.5),
  };
  let vanilla: WorldSoundEmitter = rain(XrayEngine::Vanilla, frame).expect("it rains");
  let extended: WorldSoundEmitter = rain(XrayEngine::Extended, frame).expect("it rains");

  assert!((vanilla.volume - 0.1 * 0.5).abs() < 1e-3, "{}", vanilla.volume);
  assert!(
    (extended.volume - (0.05 * 0.5 + 0.6 * 0.5) * 0.5).abs() < 1e-3,
    "{}",
    extended.volume
  );
  assert_eq!([extended.min_distance, extended.max_distance], [20.0, 40.0]);
}
