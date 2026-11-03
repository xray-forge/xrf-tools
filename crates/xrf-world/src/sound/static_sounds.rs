use std::time::Duration;

use xrf_engine_target::XrayEngine;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_static_sound::WorldStaticSound;
use crate::sound::sound_emitters::{SoundEmitters, SoundPlay};
use crate::sound::sound_random::SoundRandom;

/// What the static sounds' schedule draws from on every level open.
const SEED: u64 = 0x57A7_1C50;

/// A level's static sounds as `SStaticSound::Update` plays them each frame: within its active window and not playing,
/// one with no pause loops for ever; any other plays once its next time comes, through once where it has no play
/// window, else looped until a drawn stop, and waits a drawn pause after; one playing past its stop, or outside its
/// window, stops deferred.
pub struct StaticSounds {
  sounds: Vec<StaticSound>,
  random: SoundRandom,
}

struct StaticSound {
  record: WorldStaticSound,
  emitter: Option<u64>,
  /// `m_NextTime` and `m_StopTime` on the level's clock; no stop time plays for ever.
  next: Duration,
  stop: Option<Duration>,
}

impl StaticSounds {
  pub fn new(records: Vec<WorldStaticSound>) -> Self {
    Self {
      sounds: records
        .into_iter()
        .map(|record| StaticSound {
          record,
          emitter: None,
          next: Duration::ZERO,
          stop: None,
        })
        .collect(),
      random: SoundRandom::new(SEED),
    }
  }

  pub fn len(&self) -> usize {
    self.sounds.len()
  }

  /// The records, in the order the level holds them.
  pub fn iter_records(&self) -> impl Iterator<Item = &WorldStaticSound> {
    self.sounds.iter().map(|sound| &sound.record)
  }

  /// Plays each on to `now`, the level's clock, at `game_time`, milliseconds of the day.
  pub fn update(&mut self, emitters: &mut SoundEmitters, engine: XrayEngine, game_time: u32, now: Duration) {
    for sound in &mut self.sounds {
      let record: &WorldStaticSound = &sound.record;
      let is_playing: bool = sound.emitter.is_some_and(|id| emitters.is_playing(id, now));

      if !Self::is_active(record.active, game_time) {
        if is_playing && let Some(id) = sound.emitter {
          emitters.stop_deferred(id, now);
        }

        continue;
      }

      if is_playing {
        if let (Some(id), Some(stop)) = (sound.emitter, sound.stop)
          && now >= stop
        {
          emitters.stop_deferred(id, now);
        }

        continue;
      }

      let play = |is_looped: bool| SoundPlay {
        is_looped,
        volume: record.volume,
        frequency: record.frequency,
        ..SoundPlay::new(&record.sound, WorldSoundKind::Static, record.position, now)
      };

      if record.pause == [0, 0] {
        sound.emitter = emitters.play(engine, play(true));
        sound.stop = None;
      } else if now >= sound.next {
        let is_full_play: bool = record.play == [0, 0];

        sound.emitter = emitters.play(engine, play(!is_full_play));

        if is_full_play {
          // The file's own length in whole milliseconds, whatever speed it plays at.
          let length: Duration = record.sound.info.map_or(Duration::ZERO, |info| {
            Duration::from_millis(info.length.as_millis() as u64)
          });

          sound.stop = None;
          sound.next = now + length + self.random.millis(record.pause);
        } else {
          let stop: Duration = now + self.random.millis(record.play);

          sound.stop = Some(stop);
          sound.next = stop + self.random.millis(record.pause);
        }
      }
    }
  }

  /// Whether `game_time` falls in a window, `[from, to)`; both nought is always.
  fn is_active([from, to]: [u32; 2], game_time: u32) -> bool {
    (from == 0 && to == 0) || (game_time >= from && game_time < to)
  }
}
