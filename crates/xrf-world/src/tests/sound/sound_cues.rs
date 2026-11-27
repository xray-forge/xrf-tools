use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use xrf_audio_scene::{AudioBus, AudioSceneEmitter, AudioSceneFrame};
use xrf_math::glam::Vec3;
use xrf_renderer::{RenderSceneId, RenderViewportId, RenderWorkers};

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::sink::world_sound_frame::WorldSoundFrame;
use crate::host::sound::sink::world_sound_sink::WorldSoundSink;
use crate::host::sound::world_sound_cue::WorldSoundCue;
use crate::host::sound::world_sound_cues::WorldSoundCues;
use crate::host::world_level_source::WorldLevelSource;
use crate::level::effects::campfire::Campfire;
use crate::level::effects::level_campfires::LevelCampfires;
use crate::sound::level::level_sound::LevelSound;
use crate::sound::level::sound_drive::SoundDrive;
use crate::sound::schedule::zone_states::ZoneStates;
use crate::sound::weather::rain_frame::RainFrame;
use crate::tests::fixtures::{SilentSink, TestLevelSource};
use crate::tests::test_workers::create_workers;
use crate::world::World;

#[derive(Clone, Default)]
struct KeepingSink {
  sent: Arc<Mutex<Vec<WorldSoundFrame>>>,
}

impl WorldSoundSink for KeepingSink {
  fn send(&mut self, frame: WorldSoundFrame) {
    self.sent.lock().unwrap().push(frame);
  }

  fn take_spent(&mut self) -> Option<Box<AudioSceneFrame>> {
    None
  }
}

struct NoZones(LevelCampfires);

impl ZoneStates for NoZones {
  fn get_campfire(&mut self, id: u16) -> Campfire {
    *self.0.get(id)
  }

  fn get_place(&mut self, _: &str) -> Option<Vec3> {
    None
  }
}

fn advance(sound: &mut LevelSound, zones: &mut NoZones, opened: Instant, now: Duration) {
  let drive: SoundDrive<'_> = SoundDrive {
    scene: RenderSceneId(1),
    at: opened + now,
    is_audible: true,
    is_room_heard: true,
    eye: Vec3::ZERO,
    forward: Vec3::Z,
    up: Vec3::Y,
    game_time: 12 * 3_600_000,
    toggles: Default::default(),
    ambients: None,
    rain: RainFrame::default(),
    thunder: &[],
    burst: None,
  };

  sound.advance(now, None, &drive, None, zones);
}

fn step() -> WorldSoundCue {
  WorldSoundCue {
    reference: String::from("material\\dirt\\step\\dirt1"),
    kind: WorldSoundKind::Footstep,
    position: [1.0, 2.0, 3.0],
    is_2d: true,
    volume: 0.5,
  }
}

#[test]
fn opens_cues_only_for_a_viewport_showing_a_level_and_takes_them_after_it_is_let_go() {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.attach(viewport, Box::new(SilentSink));

  assert!(world.open_sound_cues(viewport).is_none());

  world.show_level(viewport, Some(Arc::new(TestLevelSource::default())));

  let mut cues: WorldSoundCues = world.open_sound_cues(viewport).expect("the level's cues");

  world.show_level(viewport, None);
  // The level is gone: the cue has nowhere to play, and playing it is no fault.
  cues.play(step());
}

#[test]
fn plays_a_hosts_cue_once_from_the_listener_on_the_world_bus() {
  let sink: KeepingSink = KeepingSink::default();
  let sent: Arc<Mutex<Vec<WorldSoundFrame>>> = Arc::clone(&sink.sent);
  let source: Arc<dyn WorldLevelSource> = Arc::new(TestLevelSource {
    sound_sink: Mutex::new(Some(Box::new(sink))),
    ..TestLevelSource::default()
  });
  let workers: RenderWorkers = create_workers();
  let mut sound: LevelSound = LevelSound::start(&source, &workers);
  let mut zones: NoZones = NoZones(LevelCampfires::default());
  let opened: Instant = Instant::now();

  let mut cues: WorldSoundCues = sound.open_cues(&source);

  // Played from a thread of the host's own, which holds nothing of the level's but its cues.
  std::thread::spawn(move || cues.play(step()))
    .join()
    .expect("the cue played");
  advance(&mut sound, &mut zones, opened, Duration::from_millis(100));

  let frames = sent.lock().unwrap();
  let played: Vec<&AudioSceneEmitter> = frames
    .last()
    .expect("a frame")
    .sound
    .emitters
    .iter()
    .filter(|emitter| &*emitter.sound == "material\\dirt\\step\\dirt1")
    .collect();

  assert_eq!(played.len(), 1);
  assert!(played[0].is_2d);
  assert_eq!(played[0].volume, 0.5);
  assert_eq!(played[0].bus, AudioBus::World);
  assert_eq!(played[0].start, Duration::from_millis(100));

  drop(frames);
  advance(&mut sound, &mut zones, opened, Duration::from_millis(200));

  // Played once: the next frame carries it only while it plays, never a second start.
  let frames = sent.lock().unwrap();
  let starts: usize = frames
    .last()
    .expect("a frame")
    .sound
    .emitters
    .iter()
    .filter(|emitter| &*emitter.sound == "material\\dirt\\step\\dirt1")
    .count();

  assert_eq!(starts, 1);
}
