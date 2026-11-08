use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use xrf_audio_scene::AudioSceneEmitter;
use xrf_engine_target::XrayEngine;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_sound_info::WorldSoundInfo;
use crate::sound::emitters::live_emitter::LiveEmitter;
use crate::sound::emitters::sound_play::SoundPlay;

/// The ids emitters take, unique across every scene and every restart of one, so the audio never mistakes a new play
/// for an old one.
static NEXT_EMITTER: AtomicU64 = AtomicU64::new(1);

/// How long a deferred stop fades before the engine's emitter lets go of its sound: `fade_volume` falls by ten a second
/// from one (`update_culling`).
const STOP_FADE: Duration = Duration::from_millis(100);

/// What a scene plays: each sound started and not yet over, by when it ends on the level's clock, which is what says
/// whether it still plays (`_feedback()`), never what was heard.
#[derive(Default)]
pub(crate) struct SoundEmitters {
  /// Ascending by id, as ids only grow and each play is pushed last, so an emitter is found by halving rather than by
  /// walking every one: each schedule asks after its own emitters every frame.
  live: Vec<LiveEmitter>,
}

impl SoundEmitters {
  /// Starts a sound, answering its emitter, none for a sound whose file is missing, which the engine plays as nothing.
  pub(crate) fn play(&mut self, engine: XrayEngine, play: SoundPlay<'_>) -> Option<u64> {
    let info: WorldSoundInfo = play.sound.info?;
    let id: u64 = NEXT_EMITTER.fetch_add(1, Ordering::Relaxed);
    // A two-channel file always plays 2D, from the listener (`set_position`, `switch_to_2D`).
    let is_2d: bool = play.is_2d || info.is_stereo;
    let position: [f32; 3] = if info.is_stereo { [0.0; 3] } else { play.position };
    let frequency: f32 = if play.frequency > 0.0 { play.frequency } else { 1.0 };
    let [min_distance, max_distance] = play.range.unwrap_or([info.min_distance, info.max_distance]);

    self.live.push(LiveEmitter {
      emitter: AudioSceneEmitter {
        id,
        sound: Arc::clone(&play.sound.reference),
        bus: play.kind.get_category().get_bus(),
        position,
        start: play.start,
        offset: play.offset,
        stop: None,
        is_looped: play.is_looped,
        is_2d,
        volume: play.volume,
        frequency,
        min_distance,
        max_distance,
        base_volume: info.base_volume,
        // Monolith plays its music `sm_Intro`, which sends nothing to the reverb.
        has_reverb: !(play.kind == WorldSoundKind::Music && engine == XrayEngine::Extended),
      },
      kind: play.kind,
      object: play.object,
      // `fTimeToStop`: what is left of the file at the sound's speed.
      end: (!play.is_looped).then(|| play.start + info.length.saturating_sub(play.offset).div_f32(frequency)),
      is_new: true,
    });

    Some(id)
  }

  /// Whether the emitter still plays at `now`, `_feedback()`: started, not played out, and not faded out after a stop.
  pub(crate) fn is_playing(&self, id: u64, now: Duration) -> bool {
    self.find(id).is_some_and(|live| Self::is_alive(live, now))
  }

  /// `stop_deferred`: fades it out from `now`; one stopping already fades on.
  pub(crate) fn stop_deferred(&mut self, id: u64, now: Duration) {
    if let Some(live) = self.find_mut(id)
      && live.emitter.stop.is_none()
    {
      live.emitter.stop = Some(now);
    }
  }

  /// `stop`: ends it at once.
  pub(crate) fn stop(&mut self, id: u64) {
    if let Ok(index) = self.live.binary_search_by_key(&id, |live| live.emitter.id) {
      self.live.remove(index);
    }
  }

  pub(crate) fn set_volume(&mut self, id: u64, volume: f32) {
    if let Some(live) = self.find_mut(id) {
      live.emitter.volume = volume;
    }
  }

  /// `set_position`: moves a placed sound, in engine space; a 2D one stays at the listener.
  pub(crate) fn set_position(&mut self, id: u64, position: [f32; 3]) {
    if let Some(live) = self.find_mut(id)
      && !live.emitter.is_2d
    {
      live.emitter.position = position;
    }
  }

  /// Lets go of what no longer plays at `now`, keeping what no frame has carried yet.
  pub(crate) fn retire(&mut self, now: Duration) {
    self.live.retain(|live| live.is_new || Self::is_alive(live, now));
  }

  /// Every emitter a frame at `now` carries: what plays, and what started since the last frame however short; each is
  /// marked carried.
  pub(crate) fn fill(&mut self, out: &mut Vec<AudioSceneEmitter>, now: Duration) {
    for live in &mut self.live {
      if live.is_new || Self::is_alive(live, now) {
        out.push(live.emitter.clone());
      }

      live.is_new = false;
    }
  }

  /// What plays at `now`.
  pub(crate) fn iter_playing(&self, now: Duration) -> impl Iterator<Item = &LiveEmitter> {
    self.live.iter().filter(move |live| Self::is_alive(live, now))
  }

  /// A delayed sound plays from its start, as the engine's emitter holds the sound while it waits.
  fn is_alive(live: &LiveEmitter, now: Duration) -> bool {
    live.end.is_none_or(|end| now < end) && live.emitter.stop.is_none_or(|stop| now < stop + STOP_FADE)
  }

  fn find(&self, id: u64) -> Option<&LiveEmitter> {
    let index: usize = self.live.binary_search_by_key(&id, |live| live.emitter.id).ok()?;

    self.live.get(index)
  }

  fn find_mut(&mut self, id: u64) -> Option<&mut LiveEmitter> {
    let index: usize = self.live.binary_search_by_key(&id, |live| live.emitter.id).ok()?;

    self.live.get_mut(index)
  }
}
