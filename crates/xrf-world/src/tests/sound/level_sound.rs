//! A level's sound as a whole: it waits for its scene to stand ready and runs on the scene's clock, and a burst starts
//! every schedule afresh at its own time, so bursts from one time send the same frames whatever played before them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_audio_scene::{AudioBus, AudioSceneFrame};
use xrf_engine_target::XrayEngine;
use xrf_renderer::{
  RenderSceneFeedback, RenderSceneFrame, RenderSceneId, RenderViewBurst, RenderViewportId, RenderWorkers, RenderWorld,
};

use crate::host::sound::level::world_level_sounds::WorldLevelSounds;
use crate::host::sound::level::world_music_track::WorldMusicTrack;
use crate::host::sound::level::world_static_sound::WorldStaticSound;
use crate::host::sound::sink::world_sound_frame::WorldSoundFrame;
use crate::host::sound::sink::world_sound_sink::WorldSoundSink;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::level::effects::campfire::Campfire;
use crate::level::effects::level_campfires::LevelCampfires;
use crate::sound::level::level_sound::LevelSound;
use crate::sound::level::sound_drive::SoundDrive;
use crate::sound::schedule::zone_states::ZoneStates;
use crate::sound::weather::rain_frame::RainFrame;
use crate::sound::weather::thunder_sound::ThunderSound;
use crate::tests::fixtures::{SilentSink, TestLevelSource};
use crate::tests::sound::found;
use crate::tests::test_workers::create_workers;
use crate::world::World;

/// A sink keeping a copy of every frame it is sent and handing each straight back.
#[derive(Clone, Default)]
struct KeepingSink {
  sent: Arc<Mutex<Vec<WorldSoundFrame>>>,
  #[expect(clippy::vec_box, reason = "keeps the frames boxed as the sink hands them over")]
  spent: Vec<Box<AudioSceneFrame>>,
}

impl WorldSoundSink for KeepingSink {
  fn send(&mut self, frame: WorldSoundFrame) {
    self.sent.lock().unwrap().push(frame.clone());
    self.spent.push(frame.sound);
  }

  fn take_spent(&mut self) -> Option<Box<AudioSceneFrame>> {
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
  emitters: Vec<(String, AudioBus, Duration, Duration, Option<Duration>, bool)>,
}

impl From<&WorldSoundFrame> for Played {
  fn from(frame: &WorldSoundFrame) -> Self {
    let mut emitters: Vec<_> = frame
      .sound
      .emitters
      .iter()
      .map(|emitter| {
        (
          emitter.sound.to_string(),
          emitter.bus,
          emitter.start,
          emitter.offset,
          emitter.stop,
          emitter.is_looped,
        )
      })
      .collect();

    emitters.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)));

    Self {
      time: frame.sound.time,
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
  /// Where its camera looks.
  forward: Vec3,
  zones: NoZones,
  /// The workers its sounds are read on, kept while it plays.
  workers: RenderWorkers,
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
      forward: Vec3::Z,
      zones: NoZones(LevelCampfires::default()),
      workers,
    }
  }

  /// Plays from `from`, a tenth of a second at a time, until its last frame is as `is_heard` wants, as its sounds are
  /// read on a worker first; answers the time it reached.
  fn play_until(&mut self, from: f64, is_heard: impl Fn(&WorldSoundFrame) -> bool) -> f64 {
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    let mut at: f64 = from;

    while self.sent.lock().unwrap().last().is_none_or(|frame| !is_heard(frame)) {
      assert!(Instant::now() < deadline, "the level's sounds were never read");
      self.play(at, 0.1);
      at += 0.1;
    }

    at
  }

  /// Plays a frame at `now` on the level's clock, a burst's where `burst` says.
  fn advance(&mut self, now: Duration, burst: Option<RenderViewBurst>) {
    let drive: SoundDrive<'_> = SoundDrive {
      scene: RenderSceneId(1),
      at: self.opened + now,
      is_audible: true,
      eye: Vec3::ZERO,
      forward: self.forward,
      up: Vec3::Y,
      game_time: 12 * 3_600_000,
      toggles: Default::default(),
      ambients: None,
      rain: RainFrame::default(),
      thunder: &[],
      burst,
    };

    self.sound.advance(now, None, &drive, None, &mut self.zones);
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

// The level is shown two seconds before its first frame, as a long frame or a held lock delays it, and its scene stands
// ready two seconds into its frames: the scene is told its clock began at the first frame, nothing is heard before it
// stands ready, and then its sound starts on that clock, two seconds in, not four.
#[test]
fn starts_the_level_sound_once_its_scene_stands_ready_on_the_scenes_clock() {
  let sink: KeepingSink = KeepingSink::default();
  let sent: Arc<Mutex<Vec<WorldSoundFrame>>> = Arc::clone(&sink.sent);
  let source: Arc<dyn WorldLevelSource> = Arc::new(TestLevelSource {
    sounds: sounds(),
    sound_sink: Mutex::new(Some(Box::new(sink))),
    ..TestLevelSource::default()
  });
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.attach(viewport, Box::new(SilentSink));
  world.set_listener(viewport);

  let scene: RenderSceneId = world.show_level(viewport, Some(source)).expect("the level is shown");
  let first: Instant = Instant::now() + Duration::from_secs(2);
  let play = |world: &mut World, frame: u32, has_stood_ready: bool| -> Instant {
    let feedback: RenderSceneFeedback = RenderSceneFeedback {
      has_stood_ready,
      ..RenderSceneFeedback::default()
    };
    let frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(
      first + Duration::from_secs_f64(f64::from(frame) / 60.0),
      HashMap::from([(scene, feedback)]),
    );

    frames[&scene].opened
  };

  for frame in 0..120 {
    assert_eq!(play(&mut world, frame, false), first);
  }

  assert!(
    sent.lock().unwrap().is_empty(),
    "the level is heard before it stands ready"
  );

  let deadline: Instant = Instant::now() + Duration::from_secs(5);
  let mut frame: u32 = 120;

  // Its sounds are read on a worker meanwhile, maybe not yet by the frame it stands ready.
  while sent.lock().unwrap().iter().all(|frame| frame.sound.emitters.is_empty()) {
    assert!(Instant::now() < deadline, "the level's sounds were never read");
    assert_eq!(play(&mut world, frame, true), first);
    frame += 1;
  }

  let sent = sent.lock().unwrap();

  assert_eq!(sent[0].sound.time, Duration::from_secs(2));
  assert!(
    sent
      .iter()
      .flat_map(|frame| &frame.sound.emitters)
      .all(|emitter| emitter.start >= Duration::from_secs(2))
  );
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
    .is_none_or(|frame| frame.sound.emitters.is_empty())
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
      .sound
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

// A camera pitched down holds its up square to where it looks, as its view does, so the listener still has a whole
// right ear; looking straight down, it keeps the up it last had.
#[test]
fn hears_from_the_cameras_own_up_as_it_pitches() {
  let mut heard: Heard = Heard::new();
  let listened = |heard: &mut Heard, at: f64| -> (Vec3, Vec3) {
    heard.advance(Duration::from_secs_f64(at), None);

    let sent: WorldSoundFrame = heard.sent.lock().unwrap().last().cloned().expect("a frame was sent");

    (
      Vec3::from(sent.sound.listener.forward),
      Vec3::from(sent.sound.listener.up),
    )
  };

  heard.forward = Vec3::new(0.0, -1.0, 1.0);

  let (forward, up) = listened(&mut heard, 1.0);

  assert!(forward.dot(up).abs() < 1e-5, "{forward} {up}");
  assert!((up.length() - 1.0).abs() < 1e-5 && up.y > 0.0 && up.z > 0.0, "{up}");
  assert!((up.cross(forward).length() - 1.0).abs() < 1e-5);

  heard.forward = Vec3::NEG_Y;

  let (forward, down) = listened(&mut heard, 1.1);

  assert!(forward.dot(down).abs() < 1e-5, "{forward} {down}");
  assert!((down - Vec3::Z).length() < 1e-5, "{down}");
}

// A strike is heard on the level's clock as long after it struck as the sound takes to come, `distance / 300` seconds,
// over half to twice its distance: one struck 600 m away a tenth of a second before the frame at 5 s starts at 6.9 s.
#[test]
fn hears_a_strike_on_the_levels_clock_as_far_after_as_it_struck() {
  let mut heard: Heard = Heard::new();
  let weather: WorldLevelWeather = WorldLevelWeather {
    thunder_sounds: HashMap::from([(String::from("bolt"), found("ambient\\thunder", 4))]),
    ..WorldLevelWeather::default()
  };
  let now: Duration = Duration::from_secs(5);
  let at: Instant = heard.opened + now;
  let strikes: [ThunderSound; 1] = [ThunderSound {
    bolt: String::from("bolt"),
    position: Vec3::new(0.0, 100.0, 600.0),
    distance: 600.0,
    at: at - Duration::from_millis(100),
  }];
  let drive: SoundDrive<'_> = SoundDrive {
    scene: RenderSceneId(1),
    at,
    is_audible: true,
    eye: Vec3::ZERO,
    forward: Vec3::Z,
    up: Vec3::Y,
    game_time: 12 * 3_600_000,
    toggles: Default::default(),
    ambients: None,
    rain: RainFrame::default(),
    thunder: &strikes,
    burst: None,
  };

  heard.sound.advance(now, Some(&weather), &drive, None, &mut heard.zones);

  let frame: WorldSoundFrame = heard.sent.lock().unwrap().last().cloned().expect("a frame was sent");
  let thunder = frame
    .sound
    .emitters
    .iter()
    .find(|emitter| &*emitter.sound == "ambient\\thunder")
    .expect("the strike thunders");

  assert_eq!(thunder.start, Duration::from_millis(6900));
  assert_eq!([thunder.min_distance, thunder.max_distance], [300.0, 1200.0]);
}

// A reload whose spawn changed reads the level's sounds again: what it read before plays on until the next are read,
// then every schedule starts afresh from those at the clock's time, so a sound gone from the level stops.
#[test]
fn plays_the_sounds_read_again_on_a_reload() {
  let mut heard: Heard = Heard::new();
  let reached: f64 = heard.play_until(0.0, |frame| !frame.sound.emitters.is_empty());
  let named = |frame: &WorldSoundFrame| -> Vec<String> {
    let mut named: Vec<String> = frame
      .sound
      .emitters
      .iter()
      .map(|emitter| emitter.sound.to_string())
      .collect();

    named.sort_unstable();
    named.dedup();
    named
  };
  let wind = WorldLevelSounds {
    statics: sounds().statics.into_iter().take(1).collect(),
    music: Vec::new(),
    ..sounds()
  };
  let source: Arc<dyn WorldLevelSource> = Arc::new(TestLevelSource {
    sounds: wind,
    ..TestLevelSource::default()
  });
  let reloaded: Duration = Duration::from_secs_f64(reached);

  heard.sound.reload(&source, &heard.workers);
  heard.play_until(reached, |frame| named(frame) == ["ambient\\wind"]);

  let after: WorldSoundFrame = heard.sent.lock().unwrap().last().cloned().expect("a frame was sent");

  assert!(
    after.sound.emitters.iter().all(|emitter| emitter.start >= reloaded),
    "{:?}",
    Played::from(&after)
  );
}
