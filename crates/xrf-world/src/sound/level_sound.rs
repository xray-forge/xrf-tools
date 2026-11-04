use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_engine_target::XrayEngine;
use xrf_renderer::RenderWorkers;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::contract::sound::world_sound_refusal::WorldSoundRefusal;
use crate::contract::sound::world_sound_report::WorldSoundReport;
use crate::contract::sound::world_sound_voice_report::WorldSoundVoiceReport;
use crate::host::sound::world_level_sounds::WorldLevelSounds;
use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_listener::WorldSoundListener;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::level::loading::loader_once::LoaderOnce;
use crate::sound::ambient_channels::AmbientChannels;
use crate::sound::level_music::LevelMusic;
use crate::sound::rain_sound::RainSound;
use crate::sound::sound_drive::SoundDrive;
use crate::sound::sound_emitters::{SoundEmitters, SoundPlay};
use crate::sound::sound_outlet::SoundOutlet;
use crate::sound::static_sounds::StaticSounds;

/// An ambient effect's sound starting this frame: the effect's section and where it plays, in engine space.
pub struct EffectSound<'a> {
  pub effect: &'a str,
  pub sound: &'a WorldSound,
  pub position: Vec3,
}

/// A level's sound as the world plays it beside its picture, by its own clock: its static sounds and music read on a
/// loader thread, the weather's ambient channels, rain, thunder and ambient-effect sounds as the viewport it is heard
/// by plays them, and a frame to the host's audio at most sixty times a second.
pub struct LevelSound {
  pending: LoaderOnce<WorldLevelSounds>,
  /// The level's own sounds once read.
  level: Option<LevelSounds>,
  channels: AmbientChannels,
  rain: RainSound,
  /// Each ambient effect's sound last played, which the same effect started again plays over.
  effects: HashMap<String, u64>,
  emitters: SoundEmitters,
  outlet: Option<SoundOutlet>,
  /// The level's clock, and the listener, as of the last frame.
  now: Duration,
  listener: WorldSoundListener,
  is_audible: bool,
  game_time: u32,
}

struct LevelSounds {
  engine: XrayEngine,
  statics: StaticSounds,
  music: LevelMusic,
}

impl LevelSound {
  pub fn start(source: &Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    let reader: Arc<dyn WorldLevelSource> = Arc::clone(source);

    Self {
      pending: LoaderOnce::spawn(workers, "sounds", move || {
        reader
          .read_sounds()
          .inspect_err(|error| log::warn!("The level's sounds cannot be played: {error}"))
          .ok()
      }),
      level: None,
      channels: AmbientChannels::default(),
      rain: RainSound::default(),
      effects: HashMap::new(),
      emitters: SoundEmitters::default(),
      outlet: source.open_sound().map(SoundOutlet::new),
      now: Duration::ZERO,
      listener: WorldSoundListener::default(),
      is_audible: false,
      game_time: 0,
    }
  }

  /// Plays on to `now`, the level's clock started at `opened`, as `drive` gives the frame over what the level's weathers
  /// play with, an ambient effect's sound starting with it where one does, and sends the scene's frame where one is due.
  pub fn advance(
    &mut self,
    now: Duration,
    opened: Instant,
    weather: Option<&WorldLevelWeather>,
    drive: &SoundDrive<'_>,
    effect: Option<EffectSound<'_>>,
  ) {
    let delta: Duration = now.saturating_sub(self.now);

    self.take_read();
    self.follow_listener(drive, delta);
    self.now = now;
    self.is_audible = drive.is_audible;
    self.game_time = drive.game_time;

    let engine: XrayEngine = match (&self.level, weather) {
      (Some(level), _) => level.engine,
      (None, Some(weather)) => weather.engine,
      (None, None) => XrayEngine::Vanilla,
    };

    if let Some(level) = &mut self.level {
      level
        .statics
        .update(&mut self.emitters, level.engine, drive.game_time, now);
      level.music.update(&mut self.emitters, drive.game_time, now);
    }

    if let Some(weather) = weather {
      if let Some(ambients) = drive.ambients {
        self
          .channels
          .update(&mut self.emitters, weather, ambients, drive.eye, now);
      }

      self.rain.update(
        &mut self.emitters,
        weather.rain_sound.as_ref(),
        engine,
        drive.rain,
        delta,
        now,
      );
      self.thunder(weather, drive, opened);
    }

    if let Some(effect) = effect {
      self.play_effect(engine, effect, now);
    }

    self.emitters.retire(now);
    self.send(drive, engine);
  }

  /// What the scene plays as the world schedules it; the host's audio fills how each is heard.
  pub fn report(&self, weather: Option<&WorldLevelWeather>) -> WorldSoundReport {
    let listener: Vec3 = Vec3::from(self.listener.position);

    WorldSoundReport {
      is_audible: self.is_audible,
      listener: self.listener.position,
      game_time: self.game_time,
      voices: self
        .emitters
        .iter_playing(self.now)
        .map(|emitter| WorldSoundVoiceReport {
          id: emitter.id,
          sound: emitter.sound.to_string(),
          kind: emitter.kind,
          category: emitter.kind.get_category(),
          position: emitter.position,
          is_2d: emitter.is_2d,
          distance: (!emitter.is_2d).then(|| listener.distance(Vec3::from(emitter.position))),
          gain: None,
          occlusion: None,
          transmission: None,
          reverb_send: None,
          is_binaural: false,
          is_real: false,
          is_loading: false,
          restarts: 0,
        })
        .collect(),
      music: self
        .level
        .as_ref()
        .map(|level| level.music.report(&self.emitters, self.now)),
      ambient: self.channels.get_ambient().map(str::to_owned),
      channels: self.channels.report(weather, self.now),
      refused: self.list_missing(weather),
      ..WorldSoundReport::default()
    }
  }

  fn take_read(&mut self) {
    if let Some(sounds) = self.pending.take() {
      log::info!(
        "Level sounds: {} static sounds, {} music tracks",
        sounds.statics.len(),
        sounds.music.len()
      );

      self.level = Some(LevelSounds {
        engine: sounds.engine,
        statics: StaticSounds::new(sounds.statics),
        music: LevelMusic::new(sounds.music, sounds.engine),
      });
    }
  }

  /// Hears the frame from the drive's camera, moving as fast as it moved since the last.
  fn follow_listener(&mut self, drive: &SoundDrive<'_>, delta: Duration) {
    let before: Vec3 = Vec3::from(self.listener.position);
    let seconds: f32 = delta.as_secs_f32();
    let velocity: Vec3 = if seconds > 0.0 && self.now > Duration::ZERO {
      (drive.eye - before) / seconds
    } else {
      Vec3::ZERO
    };

    self.listener = WorldSoundListener {
      position: drive.eye.to_array(),
      forward: drive.forward.normalize_or(Vec3::Z).to_array(),
      up: drive.up.normalize_or(Vec3::Y).to_array(),
      velocity: velocity.to_array(),
    };
  }

  /// Each strike's sound, delayed by how far it struck.
  fn thunder(&mut self, weather: &WorldLevelWeather, drive: &SoundDrive<'_>, opened: Instant) {
    for strike in drive.thunder {
      let Some(sound) = weather.thunder_sounds.get(&strike.bolt) else {
        continue;
      };
      let struck: Duration = strike.at.saturating_duration_since(opened);

      self.emitters.play(
        weather.engine,
        SoundPlay {
          range: Some(strike.get_range()),
          ..SoundPlay::new(
            sound,
            WorldSoundKind::Thunder,
            strike.position.to_array(),
            struck + strike.get_delay(),
          )
        },
      );
    }
  }

  /// `eff->sound.play_at_pos`: the effect's own sound, started over where it still plays.
  fn play_effect(&mut self, engine: XrayEngine, effect: EffectSound<'_>, now: Duration) {
    if let Some(previous) = self.effects.remove(effect.effect) {
      self.emitters.stop(previous);
    }

    let play: SoundPlay<'_> = SoundPlay::new(
      effect.sound,
      WorldSoundKind::AmbientEffect,
      effect.position.to_array(),
      now,
    );

    if let Some(id) = self.emitters.play(engine, play) {
      self.effects.insert(effect.effect.to_owned(), id);
    }
  }

  fn send(&mut self, drive: &SoundDrive<'_>, engine: XrayEngine) {
    let now: Duration = self.now;
    let Some(outlet) = self.outlet.as_mut().filter(|outlet| outlet.is_due(now)) else {
      return;
    };
    let emitters: &mut SoundEmitters = &mut self.emitters;
    let listener: WorldSoundListener = self.listener;

    outlet.send(now, |frame| {
      frame.scene = drive.scene;
      frame.time = now;
      frame.engine = engine;
      frame.is_audible = drive.is_audible;
      frame.listener = listener;
      frame.toggles = drive.toggles;
      emitters.fill(&mut frame.emitters, now);
    });
  }

  /// The sounds the scene would play whose files are missing: its static sounds, its music, the channels of the
  /// ambient playing, and the rain.
  fn list_missing(&self, weather: Option<&WorldLevelWeather>) -> Vec<WorldSoundRefusal> {
    let mut missing: Vec<(&WorldSound, WorldSoundKind)> = Vec::new();

    if let Some(level) = &self.level {
      missing.extend(
        level
          .statics
          .iter_records()
          .map(|record| (&record.sound, WorldSoundKind::Static)),
      );
      missing.extend(
        level
          .music
          .iter_tracks()
          .filter(|track| track.halves.is_none() || level.engine == XrayEngine::Extended)
          .map(|track| (&track.stereo, WorldSoundKind::Music)),
      );
    }

    if let Some(weather) = weather {
      let channels = self
        .channels
        .get_ambient()
        .and_then(|name| weather.ambients.get(name))
        .into_iter()
        .flat_map(|ambient| &ambient.sound_channels)
        .filter_map(|name| weather.sound_channels.get(name))
        .flat_map(|channel| &channel.sounds);

      missing.extend(channels.map(|sound| (sound, WorldSoundKind::Channel)));
      missing.extend(weather.rain_sound.iter().map(|sound| (sound, WorldSoundKind::Rain)));
    }

    let mut refused: Vec<WorldSoundRefusal> = missing
      .into_iter()
      .filter(|(sound, _)| sound.info.is_none())
      .map(|(sound, kind)| WorldSoundRefusal {
        sound: sound.reference.to_string(),
        kind: Some(kind),
        reason: String::from("The sound's file is missing or not a sound the engine loads"),
      })
      .collect();

    // A missing file is named once, however many sounds name it.
    refused.sort_by(|a, b| a.sound.cmp(&b.sound));
    refused.dedup_by(|a, b| a.sound == b.sound);
    refused
  }
}
