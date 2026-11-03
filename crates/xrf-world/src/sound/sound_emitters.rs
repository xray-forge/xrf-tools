use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use xrf_engine_target::XrayEngine;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_emitter::WorldSoundEmitter;
use crate::host::sound::world_sound_info::WorldSoundInfo;

/// The ids emitters take, unique across every scene and every restart of one, so the audio never mistakes a new play
/// for an old one.
static NEXT_EMITTER: AtomicU64 = AtomicU64::new(1);

/// How long a deferred stop fades before the engine's emitter lets go of its sound: `fade_volume` falls by ten a second
/// from one (`update_culling`).
const STOP_FADE: Duration = Duration::from_millis(100);

/// What a scene plays: each sound started and not yet over, by when it ends on the level's clock, which is what says
/// whether it still plays (`_feedback()`), never what was heard.
#[derive(Default)]
pub struct SoundEmitters {
  live: Vec<LiveEmitter>,
}

struct LiveEmitter {
  emitter: WorldSoundEmitter,
  /// When it plays out, none while looped.
  end: Option<Duration>,
  /// Whether no frame has carried it yet, which keeps a short sound for the next frame even once it is over.
  is_new: bool,
}

/// How one sound starts.
pub struct SoundPlay<'a> {
  pub sound: &'a WorldSound,
  pub kind: WorldSoundKind,
  /// In engine space; from the listener for a sound played 2D.
  pub position: [f32; 3],
  /// When its first frame plays, on the level's clock.
  pub start: Duration,
  pub is_looped: bool,
  /// Whether it plays 2D whatever its file, `sm_2D`.
  pub is_2d: bool,
  pub volume: f32,
  pub frequency: f32,
  /// Metres it is heard within, in place of its file's, `set_range`.
  pub range: Option<[f32; 2]>,
}

impl<'a> SoundPlay<'a> {
  /// A sound played once at its own volume and speed, 3D unless its file is stereo.
  pub fn new(sound: &'a WorldSound, kind: WorldSoundKind, position: [f32; 3], start: Duration) -> Self {
    Self {
      sound,
      kind,
      position,
      start,
      is_looped: false,
      is_2d: false,
      volume: 1.0,
      frequency: 1.0,
      range: None,
    }
  }
}

impl SoundEmitters {
  /// Starts a sound, answering its emitter, none for a sound whose file is missing, which the engine plays as nothing.
  pub fn play(&mut self, engine: XrayEngine, play: SoundPlay<'_>) -> Option<u64> {
    let info: WorldSoundInfo = play.sound.info?;
    let id: u64 = NEXT_EMITTER.fetch_add(1, Ordering::Relaxed);
    // A two-channel file always plays 2D, from the listener (`set_position`, `switch_to_2D`).
    let is_2d: bool = play.is_2d || info.is_stereo;
    let position: [f32; 3] = if info.is_stereo { [0.0; 3] } else { play.position };
    let frequency: f32 = if play.frequency > 0.0 { play.frequency } else { 1.0 };
    let [min_distance, max_distance] = play.range.unwrap_or([info.min_distance, info.max_distance]);

    self.live.push(LiveEmitter {
      emitter: WorldSoundEmitter {
        id,
        sound: play.sound.reference.clone(),
        kind: play.kind,
        position,
        start: play.start,
        stop: None,
        is_looped: play.is_looped,
        is_2d,
        volume: play.volume,
        frequency,
        min_distance,
        max_distance,
        base_volume: info.base_volume,
        is_occluded: !info.is_world_ambient,
        // Monolith plays its music `sm_Intro`, which sends nothing to the reverb.
        has_reverb: !(play.kind == WorldSoundKind::Music && engine == XrayEngine::Extended),
      },
      // `fTimeToStop`: the file's length at the sound's speed.
      end: (!play.is_looped).then(|| play.start + info.length.div_f32(frequency)),
      is_new: true,
    });

    Some(id)
  }

  /// Whether the emitter still plays at `now`, `_feedback()`: started, not played out, and not faded out after a stop.
  pub fn is_playing(&self, id: u64, now: Duration) -> bool {
    self.find(id).is_some_and(|live| Self::is_alive(live, now))
  }

  /// `stop_deferred`: fades it out from `now`; one stopping already fades on.
  pub fn stop_deferred(&mut self, id: u64, now: Duration) {
    if let Some(live) = self.find_mut(id)
      && live.emitter.stop.is_none()
    {
      live.emitter.stop = Some(now);
    }
  }

  /// `stop`: ends it at once.
  pub fn stop(&mut self, id: u64) {
    self.live.retain(|live| live.emitter.id != id);
  }

  pub fn set_volume(&mut self, id: u64, volume: f32) {
    if let Some(live) = self.find_mut(id) {
      live.emitter.volume = volume;
    }
  }

  /// Lets go of what no longer plays at `now`, keeping what no frame has carried yet.
  pub fn retire(&mut self, now: Duration) {
    self.live.retain(|live| live.is_new || Self::is_alive(live, now));
  }

  /// Every emitter a frame at `now` carries: what plays, and what started since the last frame however short; each is
  /// marked carried.
  pub fn fill(&mut self, out: &mut Vec<WorldSoundEmitter>, now: Duration) {
    for live in &mut self.live {
      if live.is_new || Self::is_alive(live, now) {
        out.push(live.emitter.clone());
      }

      live.is_new = false;
    }
  }

  /// What plays at `now`.
  pub fn iter_playing(&self, now: Duration) -> impl Iterator<Item = &WorldSoundEmitter> {
    self
      .live
      .iter()
      .filter(move |live| Self::is_alive(live, now))
      .map(|live| &live.emitter)
  }

  /// A delayed sound plays from its start, as the engine's emitter holds the sound while it waits.
  fn is_alive(live: &LiveEmitter, now: Duration) -> bool {
    live.end.is_none_or(|end| now < end) && live.emitter.stop.is_none_or(|stop| now < stop + STOP_FADE)
  }

  fn find(&self, id: u64) -> Option<&LiveEmitter> {
    self.live.iter().find(|live| live.emitter.id == id)
  }

  fn find_mut(&mut self, id: u64) -> Option<&mut LiveEmitter> {
    self.live.iter_mut().find(|live| live.emitter.id == id)
  }
}
