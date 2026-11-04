//! A level's sound as a whole: a burst starts every schedule afresh at its own time, so bursts from one time send the
//! same frames whatever played before them.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_engine_target::XrayEngine;
use xrf_renderer::{RenderSceneId, RenderViewBurst, RenderWorkers};

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_level_sounds::WorldLevelSounds;
use crate::host::sound::world_music_track::WorldMusicTrack;
use crate::host::sound::world_sound_frame::WorldSoundFrame;
use crate::host::sound::world_sound_sink::WorldSoundSink;
use crate::host::sound::world_static_sound::WorldStaticSound;
use crate::host::world_level_source::WorldLevelSource;
use crate::level::effects::campfire::Campfire;
use crate::level::effects::level_campfires::LevelCampfires;
use crate::sound::level_sound::LevelSound;
use crate::sound::rain_sound::RainFrame;
use crate::sound::sound_drive::SoundDrive;
use crate::sound::zone_sounds::ZoneStates;
use crate::tests::fixtures::TestLevelSource;
use crate::tests::sound::found;
use crate::tests::test_workers::create_workers;

/// A sink keeping a copy of every frame it is sent and handing each straight back.
#[derive(Clone, Default)]
struct KeepingSink {
  sent: Arc<Mutex<Vec<WorldSoundFrame>>>,
  spent: Vec<WorldSoundFrame>,
}

impl WorldSoundSink for KeepingSink {
  fn send(&mut self, frame: WorldSoundFrame) {
    self.sent.lock().unwrap().push(frame.clone());
    self.spent.push(frame);
  }

  fn take_spent(&mut self) -> Option<WorldSoundFrame> {
    self.spent.pop()
  }
}

/// No zones stand in the level.
struct NoZones(LevelCampfires);

impl ZoneStates for NoZones {
  fn get_campfire(&mut self, id: u16) -> Campfire {
    *self.0.get(id)
  }

  fn get_place(&mut self, _: &str) -> Option<Vec3> {
    None
  }
}

/// What a frame plays, but for the emitters' ids, which every play takes anew.
#[derive(Debug, PartialEq)]
struct Played {
  time: Duration,
  emitters: Vec<(String, WorldSoundKind, Duration, Duration, Option<Duration>, bool)>,
}

impl From<&WorldSoundFrame> for Played {
  fn from(frame: &WorldSoundFrame) -> Self {
    let mut emitters: Vec<_> = frame
      .emitters
      .iter()
      .map(|emitter| {
        (
          emitter.sound.to_string(),
          emitter.kind,
          emitter.start,
          emitter.offset,
          emitter.stop,
          emitter.is_looped,
        )
      })
      .collect();

    emitters.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)));

    Self {
      time: frame.time,
      emitters,
    }
  }
}

/// A level whose sounds draw their times at random: a loop, a sound played in drawn windows, and music picked at random
/// between drawn pauses.
fn sounds() -> WorldLevelSounds {
  let windowed = |reference: &str| WorldStaticSound {
    sound: found(reference, 1),
    object: None,
    position: [3.0, 0.0, 4.0],
    volume: 1.0,
    frequency: 1.0,
    active: [0, 0],
    play: [500, 1500],
    pause: [700, 2300],
  };
  let track = |name: &str| WorldMusicTrack {
    name: name.to_owned(),
    stereo: found(name, 3),
    halves: None,
    hours: [0, 0],
    volume: 0.8,
    pause: [1000, 4000],
  };

  WorldLevelSounds {
    engine: XrayEngine::Vanilla,
    statics: vec![
      WorldStaticSound {
        pause: [0, 0],
        ..windowed("ambient\\wind")
      },
      windowed("ambient\\drip"),
      windowed("ambient\\creak"),
    ],
    music: ["music\\a", "music\\b", "music\\c"].map(track).to_vec(),
    zones: Vec::new(),
  }
}

/// A level's sound, heard where the frames it sends go.
struct Heard {
  sound: LevelSound,
  sent: Arc<Mutex<Vec<WorldSoundFrame>>>,
  opened: Instant,
  zones: NoZones,
  /// The workers its sounds are read on, kept while it plays.
  _workers: RenderWorkers,
}

impl Heard {
  fn new() -> Self {
    let sink: KeepingSink = KeepingSink::default();
    let sent: Arc<Mutex<Vec<WorldSoundFrame>>> = Arc::clone(&sink.sent);
    let source: Arc<dyn WorldLevelSource> = Arc::new(TestLevelSource {
      sounds: sounds(),
      sound_sink: Mutex::new(Some(Box::new(sink))),
      ..TestLevelSource::default()
    });
    let workers: RenderWorkers = create_workers();

    Self {
      sound: LevelSound::start(&source, &workers),
      sent,
      opened: Instant::now(),
      zones: NoZones(LevelCampfires::default()),
      _workers: workers,
    }
  }

  /// Plays a frame at `now` on the level's clock, a burst's where `burst` says.
  fn advance(&mut self, now: Duration, burst: Option<RenderViewBurst>) {
    let drive: SoundDrive<'_> = SoundDrive {
      scene: RenderSceneId(1),
      at: self.opened + now,
      is_audible: true,
      eye: Vec3::ZERO,
      forward: Vec3::Z,
      up: Vec3::Y,
      game_time: 12 * 3_600_000,
      toggles: Default::default(),
      ambients: None,
      rain: RainFrame::default(),
      thunder: &[],
      burst,
    };

    self
      .sound
      .advance(now, self.opened, None, &drive, None, &mut self.zones);
  }

  /// Plays the live clock from `from` for `seconds`, sixty frames a second.
  fn play(&mut self, from: f64, seconds: f64) {
    for frame in 0..(seconds * 60.0) as u32 {
      self.advance(Duration::from_secs_f64(from + f64::from(frame) / 60.0), None);
    }
  }

  /// Draws a burst of 120 frames 1/120 s apart from 600 s, answering what its frames sent.
  fn burst(&mut self) -> Vec<Played> {
    let before: usize = self.sent.lock().unwrap().len();

    for frame in 0..120u32 {
      let burst: RenderViewBurst = if frame == 0 {
        RenderViewBurst::Starting
      } else {
        RenderViewBurst::Drawing
      };

      self.advance(Duration::from_secs_f64(600.0 + f64::from(frame) / 120.0), Some(burst));
    }

    let sent = self.sent.lock().unwrap();

    assert!(sent[before..].iter().all(|frame| frame.is_burst));

    sent[before..].iter().map(Played::from).collect()
  }
}

#[test]
fn sends_the_same_frames_for_two_bursts_from_one_time() {
  let mut heard: Heard = Heard::new();
  let deadline: Instant = Instant::now() + Duration::from_secs(5);

  // The level's sounds are read on a worker first.
  while heard
    .sent
    .lock()
    .unwrap()
    .last()
    .is_none_or(|frame| frame.emitters.is_empty())
  {
    assert!(Instant::now() < deadline, "the level's sounds were never read");
    heard.play(0.0, 0.1);
  }

  heard.play(0.1, 20.0);

  let first: Vec<Played> = heard.burst();

  // Live again, the schedules start afresh: the next frame is no burst's and plays its loop from the live time.
  heard.play(20.1, 1.0 / 60.0);

  let after: WorldSoundFrame = heard.sent.lock().unwrap().last().cloned().expect("a frame was sent");

  assert!(!after.is_burst);
  assert!(
    after
      .emitters
      .iter()
      .any(|emitter| emitter.start == Duration::from_secs_f64(20.1))
  );

  heard.play(20.2, 33.0);

  let second: Vec<Played> = heard.burst();

  assert!(first.len() >= 60, "{} frames", first.len());
  assert!(first.iter().any(|played| played.emitters.len() >= 3), "{first:?}");
  assert_eq!(first, second);
}
