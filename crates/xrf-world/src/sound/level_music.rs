use std::time::Duration;

use xrf_engine_target::XrayEngine;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::contract::sound::world_sound_music_report::WorldSoundMusicReport;
use crate::host::sound::world_music_track::WorldMusicTrack;
use crate::sound::sound_emitters::{SoundEmitters, SoundPlay};
use crate::sound::sound_random::SoundRandom;

/// What the music's picks and pauses draw from on every level open.
const SEED: u64 = 0x0B5E_55ED;

/// How long the music waits to pick again when no track is eligible.
const RECHECK: Duration = Duration::from_secs(10);

/// Where OpenXRay plays a track's halves where its stereo file is missing, from the listener.
const HALF_POSITIONS: [[f32; 3]; 2] = [[-0.5, 0.0, 0.3], [0.5, 0.0, 0.3]];

/// A level's music as `CLevelSoundManager::Update` plays it: with no track playing and the level's clock past the next
/// check, every track left playing stops and one eligible by the game's hour is picked, 2D and at its volume; none
/// eligible checks again after ten seconds; once it ends, the next waits a drawn pause.
///
/// OpenXRay picks at random among the eligible, and plays a track's halves where its stereo file is missing. Monolith
/// shuffles the tracks once and takes the first eligible one not yet played; none left forgets what played and shuffles
/// again.
pub struct LevelMusic {
  tracks: Vec<WorldMusicTrack>,
  engine: XrayEngine,
  /// The track playing, by its place in `tracks`, and its emitters.
  current: Option<(usize, Vec<u64>)>,
  next: Duration,
  /// Monolith's tracks played since the last shuffle.
  played: Vec<usize>,
  random: SoundRandom,
}

impl LevelMusic {
  pub fn new(mut tracks: Vec<WorldMusicTrack>, engine: XrayEngine) -> Self {
    let mut random: SoundRandom = SoundRandom::new(SEED);

    if engine == XrayEngine::Extended {
      Self::shuffle(&mut tracks, &mut random);
    }

    Self {
      tracks,
      engine,
      current: None,
      next: Duration::ZERO,
      played: Vec::new(),
      random,
    }
  }

  /// Plays on to `now`, the level's clock, at `game_time`, milliseconds of the day.
  pub fn update(&mut self, emitters: &mut SoundEmitters, game_time: u32, now: Duration) {
    if self.tracks.is_empty() {
      return;
    }

    if self.current.is_none() && now > self.next {
      self.pick(emitters, game_time, now);
    }

    if let Some((index, playing)) = &self.current
      && !Self::is_track_playing(emitters, playing, now)
    {
      let pause: [u32; 2] = self.tracks[*index].pause;

      self.current = None;
      self.next = now;

      if pause != [0, 0] {
        self.next += self.random.millis(pause);
      }
    }
  }

  /// The track playing and how far into it, or how long until the next pick.
  pub fn report(&self, emitters: &SoundEmitters, now: Duration) -> WorldSoundMusicReport {
    let playing = self.current.as_ref().and_then(|(index, ids)| {
      emitters
        .iter_playing(now)
        .find(|emitter| ids.contains(&emitter.id))
        .map(|emitter| (self.tracks[*index].name.clone(), now.saturating_sub(emitter.start)))
    });

    WorldSoundMusicReport {
      wait: if playing.is_some() {
        Duration::ZERO
      } else {
        self.next.saturating_sub(now)
      },
      position: playing.as_ref().map_or(Duration::ZERO, |(_, position)| *position),
      track: playing.map(|(track, _)| track),
      tracks: self.tracks.len() as u32,
    }
  }

  pub fn iter_tracks(&self) -> impl Iterator<Item = &WorldMusicTrack> {
    self.tracks.iter()
  }

  fn pick(&mut self, emitters: &mut SoundEmitters, game_time: u32, now: Duration) {
    let eligible: Vec<usize> = (0..self.tracks.len())
      .filter(|index| Self::is_eligible(self.tracks[*index].hours, game_time))
      .filter(|index| self.engine == XrayEngine::Vanilla || !self.played.contains(index))
      .collect();
    let picked: Option<usize> = match self.engine {
      XrayEngine::Vanilla => (!eligible.is_empty()).then(|| eligible[self.random.index(eligible.len())]),
      XrayEngine::Extended => eligible.first().copied(),
    };

    let Some(index) = picked else {
      if self.engine == XrayEngine::Extended {
        self.played.clear();
        Self::shuffle(&mut self.tracks, &mut self.random);
      }

      self.next = now + RECHECK;

      return;
    };

    self.played.push(index);
    self.current = Some((index, self.play(emitters, index, now)));
  }

  /// `SMusicTrack::Play`: the stereo file 2D, or OpenXRay's halves either side, each at the track's volume.
  fn play(&self, emitters: &mut SoundEmitters, index: usize, now: Duration) -> Vec<u64> {
    let track: &WorldMusicTrack = &self.tracks[index];
    let play = |sound, position| SoundPlay {
      is_2d: true,
      volume: track.volume,
      ..SoundPlay::new(sound, WorldSoundKind::Music, position, now)
    };

    if track.stereo.info.is_some() {
      return emitters
        .play(self.engine, play(&track.stereo, [0.0; 3]))
        .into_iter()
        .collect();
    }

    match (&track.halves, self.engine) {
      (Some(halves), XrayEngine::Vanilla) => halves
        .iter()
        .zip(HALF_POSITIONS)
        .filter_map(|(half, position)| emitters.play(self.engine, play(half, position)))
        .collect(),
      _ => Vec::new(),
    }
  }

  /// `SMusicTrack::IsPlaying`: the stereo file, or both halves.
  fn is_track_playing(emitters: &SoundEmitters, playing: &[u64], now: Duration) -> bool {
    !playing.is_empty() && playing.iter().all(|id| emitters.is_playing(*id, now))
  }

  /// `SMusicTrack::in`: within its hours, which may cross midnight; a track from hour nought to any other is always
  /// eligible.
  fn is_eligible([from, to]: [u32; 2], game_time: u32) -> bool {
    if from == 0 && to != 0 {
      return true;
    }

    if to < from {
      game_time >= from || game_time <= to
    } else {
      game_time >= from && game_time < to
    }
  }

  /// `std::shuffle`, Fisher-Yates over the schedule's own numbers.
  fn shuffle(tracks: &mut [WorldMusicTrack], random: &mut SoundRandom) {
    for last in (1..tracks.len()).rev() {
      tracks.swap(last, random.index(last + 1));
    }
  }
}
