use std::time::Duration;

use glam::Vec3;
use xrf_engine_target::XrayEngine;
use xrf_renderer::ZONE_FAST_DISTANCE;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::level::world_zone_sound::WorldZoneSound;
use crate::host::sound::world_sound::WorldSound;
use crate::level::effects::campfire::Campfire;
use crate::sound::emitters::sound_emitters::SoundEmitters;
use crate::sound::emitters::sound_play::SoundPlay;
use crate::sound::schedule::zone_states::ZoneStates;

/// A level's zones' sounds as `CCustomZone` plays them: its `idle_sound` looped with its idle (`PlayIdleParticles`,
/// `StopIdleParticles`), a torrid zone's following its motion, and a campfire's idle through its turn's gates and its
/// `disabled_sound` looped while it is out (`CZoneCampfire`).
pub struct ZoneSounds {
  zones: Vec<ZoneSound>,
}

struct ZoneSound {
  record: WorldZoneSound,
  idle: Option<u64>,
  disabled: Option<u64>,
}

impl ZoneSounds {
  pub fn new(records: Vec<WorldZoneSound>) -> Self {
    Self {
      zones: records
        .into_iter()
        .map(|record| ZoneSound {
          record,
          idle: None,
          disabled: None,
        })
        .collect(),
    }
  }

  pub fn len(&self) -> usize {
    self.zones.len()
  }

  /// The records, in the spawn's order.
  pub fn iter_records(&self) -> impl Iterator<Item = &WorldZoneSound> {
    self.zones.iter().map(|zone| &zone.record)
  }

  /// Plays each on to `now`, the level's clock, as `states` stand its zones, heard from `eye` in engine space.
  pub fn update(
    &mut self,
    emitters: &mut SoundEmitters,
    engine: XrayEngine,
    states: &mut dyn ZoneStates,
    eye: Vec3,
    now: Duration,
  ) {
    for zone in &mut self.zones {
      let record: &WorldZoneSound = &zone.record;
      let place: Vec3 = record
        .motion
        .as_deref()
        .and_then(|motion| states.get_place(motion))
        .unwrap_or(Vec3::from(record.position));
      // Only Monolith stops a slowed zone's idle (`o_switch_2_slow`); Vanilla's slowing puts out its light alone.
      let is_far: bool = engine == XrayEngine::Extended
        && record
          .zone_sphere
          .as_ref()
          .is_some_and(|sphere| sphere.get_distance(place.to_array(), eye.to_array()) > ZONE_FAST_DISTANCE);
      let kind: WorldSoundKind = record.get_kind();
      let start = |emitters: &mut SoundEmitters, playing: &mut Option<u64>, sound: Option<&WorldSound>| {
        if let Some(sound) = sound
          && !playing.is_some_and(|id| emitters.is_playing(id, now))
        {
          *playing = emitters.play(
            engine,
            SoundPlay {
              is_looped: true,
              object: record.object,
              ..SoundPlay::new(sound, kind, place.to_array(), now)
            },
          );
        }
      };
      // The engines stop it at once; it fades out over a tenth of a second here, which ends it as soon without a click.
      let stop = |emitters: &mut SoundEmitters, playing: &mut Option<u64>| {
        if let Some(id) = playing.take() {
          emitters.stop_deferred(id, now);
        }
      };

      if record.is_campfire {
        let campfire: Campfire = states.get_campfire(record.id);

        // `GoDisabledState` loops the disabled sound from the moment it goes out; `GoEnabledState` stops it.
        if campfire.is_lit() {
          stop(emitters, &mut zone.disabled);
        } else {
          start(emitters, &mut zone.disabled, record.disabled.as_ref());
        }

        // A turn holds the idle as it holds the particles; an out campfire takes no far switch, being disabled.
        if campfire.is_lit() && !is_far {
          if campfire.can_play_idle(now) {
            start(emitters, &mut zone.idle, record.idle.as_ref());
          }
        } else if campfire.can_stop_idle(now) {
          stop(emitters, &mut zone.idle);
        }
      } else if is_far {
        stop(emitters, &mut zone.idle);
      } else {
        start(emitters, &mut zone.idle, record.idle.as_ref());
      }

      // `CTorridZone::shedule_Update` moves what it plays with it.
      for id in [zone.idle, zone.disabled].into_iter().flatten() {
        emitters.set_position(id, place.to_array());
      }
    }
  }
}
