//! A level's zones' sounds, `CCustomZone` and `CZoneCampfire`: the idle sound with the idle, the far switch per engine,
//! a torrid zone's sound moving with it, and a campfire's sounds through its switching.

use std::collections::HashMap;
use std::time::Duration;

use glam::Vec3;
use xrf_engine_target::XrayEngine;
use xrf_math::Vector3d;
use xrf_visual::ZoneSphere;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::level::world_zone_sound::WorldZoneSound;
use crate::level::effects::campfire::Campfire;
use crate::level::effects::campfire_state::CampfireState;
use crate::level::effects::level_campfires::LevelCampfires;
use crate::sound::emitters::live_emitter::LiveEmitter;
use crate::sound::emitters::sound_emitters::SoundEmitters;
use crate::sound::schedule::zone_sounds::ZoneSounds;
use crate::sound::schedule::zone_states::ZoneStates;
use crate::tests::sound::{found, missing, seconds};

/// The campfires as a view switches them and where each motion has its object.
#[derive(Default)]
struct States {
  campfires: LevelCampfires,
  places: HashMap<String, Vec3>,
}

impl ZoneStates for States {
  fn get_campfire(&mut self, id: u16) -> Campfire {
    *self.campfires.get(id)
  }

  fn get_place(&mut self, motion: &str) -> Option<Vec3> {
    self.places.get(motion).copied()
  }
}

/// An anomaly at the origin humming `anomaly\\hum`, measured to a sphere of five metres about it.
fn anomaly() -> WorldZoneSound {
  WorldZoneSound {
    id: 4,
    object: Some(40),
    position: [0.0; 3],
    idle: Some(found("anomaly\\hum", 3)),
    is_campfire: false,
    disabled: None,
    motion: None,
    zone_sphere: Some(ZoneSphere {
      offset: Vector3d::new(0.0, 0.0, 0.0),
      radius: 5.0,
    }),
  }
}

fn campfire() -> WorldZoneSound {
  WorldZoneSound {
    id: 7,
    object: Some(70),
    idle: Some(found("ambient\\fire", 3)),
    is_campfire: true,
    disabled: Some(found("ambient\\fire_out", 3)),
    ..anomaly()
  }
}

/// What plays on at `now`, those fading out after a stop aside.
fn playing(emitters: &SoundEmitters, now: Duration) -> Vec<LiveEmitter> {
  emitters
    .iter_playing(now)
    .filter(|live| live.emitter.stop.is_none())
    .cloned()
    .collect()
}

fn sounds_of(emitters: &SoundEmitters, now: Duration) -> Vec<String> {
  playing(emitters, now)
    .into_iter()
    .map(|live| live.emitter.sound.to_string())
    .collect()
}

/// The camera `metres` along x from the origin.
fn eye(metres: f32) -> Vec3 {
  Vec3::new(metres, 0.0, 0.0)
}

#[test]
fn hums_from_the_first_frame_looped_at_the_zone_as_its_object() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![anomaly()]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  zones.update(
    &mut emitters,
    XrayEngine::Vanilla,
    &mut States::default(),
    eye(10.0),
    Duration::ZERO,
  );

  let hum: Vec<LiveEmitter> = playing(&emitters, Duration::ZERO);

  assert_eq!(hum.len(), 1);
  assert!(hum[0].emitter.is_looped);
  assert_eq!(hum[0].kind, WorldSoundKind::Anomaly);
  assert_eq!(hum[0].object, Some(40));
  assert_eq!(hum[0].emitter.position, [0.0; 3]);
}

// Past a hundred metres from its sphere Monolith stops the idle, fading it out, and plays it afresh once the camera is
// back; Vanilla's slowing touches its light alone, so the same play goes on.
#[test]
fn falls_silent_past_the_fast_distance_on_monolith_only() {
  for (engine, is_heard_far) in [(XrayEngine::Vanilla, true), (XrayEngine::Extended, false)] {
    let mut zones: ZoneSounds = ZoneSounds::new(vec![anomaly()]);
    let mut emitters: SoundEmitters = SoundEmitters::default();
    let mut states: States = States::default();

    zones.update(&mut emitters, engine, &mut states, eye(100.0), Duration::ZERO);

    let near: u64 = playing(&emitters, Duration::ZERO)[0].emitter.id;

    // 104 metres from the centre is 99 outside the sphere: still fast.
    zones.update(&mut emitters, engine, &mut states, eye(104.0), seconds(1));
    assert_eq!(playing(&emitters, seconds(1))[0].emitter.id, near);

    zones.update(&mut emitters, engine, &mut states, eye(106.0), seconds(2));
    emitters.retire(seconds(2));
    assert_eq!(!playing(&emitters, seconds(2)).is_empty(), is_heard_far, "{engine:?}");
    // A stopped idle fades out over a tenth of a second rather than cuts.
    assert!(emitters.is_playing(near, Duration::from_millis(2099)), "{engine:?}");
    assert_eq!(
      emitters.is_playing(near, Duration::from_millis(2100)),
      is_heard_far,
      "{engine:?}"
    );

    zones.update(&mut emitters, engine, &mut states, eye(50.0), seconds(3));

    let back: Vec<LiveEmitter> = playing(&emitters, seconds(3));

    assert_eq!(back.len(), 1, "{engine:?}");
    assert_eq!(back[0].emitter.id == near, is_heard_far, "{engine:?}");
  }
}

#[test]
fn keeps_an_always_fast_zone_humming_however_far() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![WorldZoneSound {
    zone_sphere: None,
    ..anomaly()
  }]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  zones.update(
    &mut emitters,
    XrayEngine::Extended,
    &mut States::default(),
    eye(5000.0),
    Duration::ZERO,
  );

  assert_eq!(playing(&emitters, Duration::ZERO).len(), 1);
}

// A torrid zone's sound stands where its motion has the zone, and Monolith measures the far switch from there.
#[test]
fn moves_a_torrid_zones_sound_with_its_motion() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![WorldZoneSound {
    motion: Some(String::from("torrid_path")),
    ..anomaly()
  }]);
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut states: States = States::default();

  states
    .places
    .insert(String::from("torrid_path"), Vec3::new(1.0, 2.0, 3.0));
  zones.update(
    &mut emitters,
    XrayEngine::Extended,
    &mut states,
    eye(0.0),
    Duration::ZERO,
  );

  let id: u64 = playing(&emitters, Duration::ZERO)[0].emitter.id;

  assert_eq!(playing(&emitters, Duration::ZERO)[0].emitter.position, [1.0, 2.0, 3.0]);

  states
    .places
    .insert(String::from("torrid_path"), Vec3::new(4.0, 2.0, 3.0));
  zones.update(&mut emitters, XrayEngine::Extended, &mut states, eye(0.0), seconds(1));

  let moved: Vec<LiveEmitter> = playing(&emitters, seconds(1));

  assert_eq!(moved[0].emitter.id, id);
  assert_eq!(moved[0].emitter.position, [4.0, 2.0, 3.0]);

  // The camera stands near the zone's spawn, but the motion has carried it two hundred metres off.
  states
    .places
    .insert(String::from("torrid_path"), Vec3::new(200.0, 2.0, 3.0));
  zones.update(&mut emitters, XrayEngine::Extended, &mut states, eye(0.0), seconds(2));
  emitters.retire(seconds(2));

  assert!(playing(&emitters, seconds(2)).is_empty());
}

// Put out, a campfire loops its disabled sound at once and its idle stops half a second into the turn; lit again, the
// disabled sound stops at once and the idle waits for the turn's last second.
#[test]
fn switches_a_campfires_sounds_as_it_is_put_out_and_lit_again() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![campfire()]);
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut states: States = States::default();
  let mut step = |state: CampfireState, millis: u64| -> Vec<String> {
    let now: Duration = Duration::from_millis(millis);

    states.campfires.prepare(state, now);
    zones.update(&mut emitters, XrayEngine::Vanilla, &mut states, eye(1.0), now);
    emitters.retire(now);

    sounds_of(&emitters, now)
  };

  assert_eq!(step(CampfireState::Lit, 0), ["ambient\\fire"]);
  assert_eq!(step(CampfireState::Out, 10_000), ["ambient\\fire", "ambient\\fire_out"]);
  assert_eq!(step(CampfireState::Out, 10_400), ["ambient\\fire", "ambient\\fire_out"]);
  assert_eq!(step(CampfireState::Out, 10_600), ["ambient\\fire_out"]);
  assert_eq!(step(CampfireState::Out, 15_000), ["ambient\\fire_out"]);
  assert_eq!(step(CampfireState::Lit, 20_000), Vec::<String>::new());
  assert_eq!(step(CampfireState::Lit, 21_000), Vec::<String>::new());
  assert_eq!(step(CampfireState::Lit, 22_100), ["ambient\\fire"]);
}

#[test]
fn loops_the_disabled_sound_of_a_campfire_out_from_the_first() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![campfire()]);
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut states: States = States::default();

  states.campfires.prepare(CampfireState::Out, Duration::ZERO);
  zones.update(
    &mut emitters,
    XrayEngine::Extended,
    &mut states,
    eye(1.0),
    Duration::ZERO,
  );

  let out: Vec<LiveEmitter> = playing(&emitters, Duration::ZERO);

  assert_eq!(out.len(), 1);
  assert_eq!(&*out[0].emitter.sound, "ambient\\fire_out");
  assert_eq!(out[0].kind, WorldSoundKind::Campfire);
  assert!(out[0].emitter.is_looped);
}

// Monolith stops a lit campfire's idle past the fast distance as it does any zone's.
#[test]
fn quiets_a_far_campfire_on_monolith() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![campfire()]);
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut states: States = States::default();

  zones.update(
    &mut emitters,
    XrayEngine::Extended,
    &mut states,
    eye(1.0),
    Duration::ZERO,
  );
  assert_eq!(sounds_of(&emitters, Duration::ZERO), ["ambient\\fire"]);

  zones.update(&mut emitters, XrayEngine::Extended, &mut states, eye(500.0), seconds(1));
  emitters.retire(seconds(1));
  assert!(sounds_of(&emitters, seconds(1)).is_empty());
}

#[test]
fn plays_nothing_for_a_missing_idle_sound() {
  let mut zones: ZoneSounds = ZoneSounds::new(vec![WorldZoneSound {
    idle: Some(missing("anomaly\\gone")),
    ..anomaly()
  }]);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  zones.update(
    &mut emitters,
    XrayEngine::Vanilla,
    &mut States::default(),
    eye(1.0),
    Duration::ZERO,
  );

  assert!(playing(&emitters, Duration::ZERO).is_empty());
}
