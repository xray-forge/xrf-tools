use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use glam::Vec3;
use xrf_engine_target::XrayEngine;
use xrf_renderer::{RenderViewBurst, RenderWorkers};

use crate::contract::sound::report::world_sound_refusal::WorldSoundRefusal;
use crate::contract::sound::report::world_sound_report::WorldSoundReport;
use crate::contract::sound::report::world_sound_voice_report::WorldSoundVoiceReport;
use crate::contract::sound::world_music_command::WorldMusicCommand;
use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::level::world_level_sounds::WorldLevelSounds;
use crate::host::sound::sink::world_sound_listener::WorldSoundListener;
use crate::host::sound::world_sound::WorldSound;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::level::loading::loader_once::LoaderOnce;
use crate::sound::emitters::sound_emitters::SoundEmitters;
use crate::sound::emitters::sound_outlet::SoundOutlet;
use crate::sound::emitters::sound_play::SoundPlay;
use crate::sound::level::effect_sound::EffectSound;
use crate::sound::level::sound_drive::SoundDrive;
use crate::sound::schedule::level_music::LevelMusic;
use crate::sound::schedule::static_sounds::StaticSounds;
use crate::sound::schedule::zone_sounds::ZoneSounds;
use crate::sound::schedule::zone_states::ZoneStates;
use crate::sound::weather::ambient_channels::AmbientChannels;
use crate::sound::weather::rain_sound::RainSound;

/// A level's sound as the world plays it beside its picture, by the level's clock: its static sounds, music and zones'
/// sounds read on a loader thread, the weather's ambient channels, rain, thunder and ambient-effect sounds as the
/// viewport it is heard by plays them, and a frame to the host's audio at most sixty times a second.
///
/// Its schedules start at the first frame it plays, which its level gives it once its scene stands ready. A burst's
/// first frame starts every schedule afresh at the burst's time, as that first frame does, and so does the first frame
/// after a burst: what a burst plays depends on its own frames alone, so bursts from one time play alike.
pub struct LevelSound {
  pending: LoaderOnce<WorldLevelSounds>,
  /// The level's own sounds as read, which a schedule starting afresh plays from again.
  read: Option<WorldLevelSounds>,
  /// The level's own sounds once read.
  level: Option<LevelSounds>,
  /// Whether the last frame was a burst's.
  is_bursting: bool,
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
  zones: ZoneSounds,
}

impl LevelSounds {
  /// The schedules of the level's sounds as read, from their start.
  fn new(sounds: &WorldLevelSounds) -> Self {
    Self {
      engine: sounds.engine,
      statics: StaticSounds::new(sounds.statics.clone()),
      music: LevelMusic::new(sounds.music.clone(), sounds.engine),
      zones: ZoneSounds::new(sounds.zones.clone()),
    }
  }
}

impl LevelSound {
  pub fn start(source: &Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      pending: Self::read_sounds(source, workers),
      read: None,
      level: None,
      is_bursting: false,
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

  /// Reads the level's own sounds again from `source`, a next source of the same level, whose spawn or helpers changed:
  /// once read, every schedule starts afresh at the clock's time as a burst's end starts them, so an edited zone plays
  /// as it now stands and each sounding emitter names its helper by the helpers' new order. What plays meanwhile plays on.
  pub fn reload(&mut self, source: &Arc<dyn WorldLevelSource>, workers: &RenderWorkers) {
    // todo: Reopen the sink where the level's collision changed, so the acoustics' walls follow it; they stay as the
    // level opened, since a new sink builds Steam Audio's scene afresh, seconds on a large level.
    self.pending = Self::read_sounds(source, workers);
  }

  /// Plays on to `now` on the level's clock, as `drive` gives the frame over what the level's weathers play with and
  /// `zones` stand its zones, an ambient effect's sound starting with it where one does, and sends the scene's frame
  /// where one is due.
  pub fn advance(
    &mut self,
    now: Duration,
    weather: Option<&WorldLevelWeather>,
    drive: &SoundDrive<'_>,
    effect: Option<EffectSound<'_>>,
    zones: &mut dyn ZoneStates,
  ) {
    let delta: Duration = now.saturating_sub(self.now);

    self.take_read(now);

    if drive.burst == Some(RenderViewBurst::Starting) || (self.is_bursting && drive.burst.is_none()) {
      self.restart(now);
    }

    self.is_bursting = drive.burst.is_some();
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
      level
        .zones
        .update(&mut self.emitters, level.engine, zones, drive.eye, now);
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
      self.thunder(weather, drive, now);
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
          object: emitter.object,
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

  /// Does what a viewer asks of the music at the last frame's clock, answering whether it could; none can before the
  /// level's sounds are read.
  pub fn command_music(&mut self, command: &WorldMusicCommand) -> bool {
    self
      .level
      .as_mut()
      .is_some_and(|level| level.music.command(&mut self.emitters, command, self.now))
  }

  /// The level objects whose sounds play at the last frame, by the ids their helpers carry, ascending, each once.
  pub fn list_sounding(&self) -> Vec<u32> {
    let mut objects: Vec<u32> = self
      .emitters
      .iter_playing(self.now)
      .filter_map(|emitter| emitter.object)
      .collect();

    objects.sort_unstable();
    objects.dedup();
    objects
  }

  /// The level's own sounds read on a worker from `source`, none where they cannot be.
  fn read_sounds(source: &Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> LoaderOnce<WorldLevelSounds> {
    let reader: Arc<dyn WorldLevelSource> = Arc::clone(source);

    LoaderOnce::spawn(workers, "sounds", move || {
      reader
        .read_sounds()
        .inspect_err(|error| log::warn!("The level's sounds cannot be played: {error}"))
        .ok()
    })
  }

  /// Takes the level's sounds once read: the first time its schedules start from them, read again on a reload every
  /// schedule starts afresh at `now` from them.
  fn take_read(&mut self, now: Duration) {
    if let Some(sounds) = self.pending.take() {
      log::info!(
        "Level sounds: {} static sounds, {} music tracks, {} zones",
        sounds.statics.len(),
        sounds.music.len(),
        sounds.zones.len()
      );

      let is_reread: bool = self.read.is_some();

      self.read = Some(sounds);

      if is_reread {
        self.restart(now);
      } else {
        self.level = self.read.as_ref().map(LevelSounds::new);
      }
    }
  }

  /// Starts every schedule afresh at `now`, as the first frame it plays does, letting go of whatever plays; a track
  /// played by hand plays again from its start.
  fn restart(&mut self, now: Duration) {
    let chosen: Option<String> = self
      .level
      .as_ref()
      .and_then(|level| level.music.get_chosen())
      .map(str::to_owned);

    self.level = self.read.as_ref().map(LevelSounds::new);
    self.channels = AmbientChannels::default();
    self.rain = RainSound::default();
    self.effects.clear();
    self.emitters = SoundEmitters::default();

    if let Some(outlet) = &mut self.outlet {
      outlet.restart();
    }

    if let (Some(level), Some(track)) = (&mut self.level, chosen) {
      level
        .music
        .command(&mut self.emitters, &WorldMusicCommand::Play { track }, now);
    }
  }

  /// Hears the frame from the drive's camera, moving as fast as it moved since the last. Its up is the drive's made
  /// square to where it looks, as a camera looking at a point holds it, so a camera pitched up or down still hears left
  /// from right; looking along the drive's up, it keeps the last up it had.
  fn follow_listener(&mut self, drive: &SoundDrive<'_>, delta: Duration) {
    let before: Vec3 = Vec3::from(self.listener.position);
    let seconds: f32 = delta.as_secs_f32();
    let velocity: Vec3 = if seconds > 0.0 && self.now > Duration::ZERO {
      (drive.eye - before) / seconds
    } else {
      Vec3::ZERO
    };
    let forward: Vec3 = drive.forward.normalize_or(Vec3::Z);
    let square = |up: Vec3| (up - forward * forward.dot(up)).try_normalize();
    let up: Vec3 = square(drive.up)
      .or_else(|| square(Vec3::from(self.listener.up)))
      .or_else(|| square(Vec3::Z))
      .unwrap_or(Vec3::Y);

    self.listener = WorldSoundListener {
      position: drive.eye.to_array(),
      forward: forward.to_array(),
      up: up.to_array(),
      velocity: velocity.to_array(),
    };
  }

  /// Each strike's sound, delayed by how far it struck: a strike stamped at a moment before the frame's struck as long
  /// before `now` on the level's clock.
  fn thunder(&mut self, weather: &WorldLevelWeather, drive: &SoundDrive<'_>, now: Duration) {
    for strike in drive.thunder {
      let Some(sound) = weather.thunder_sounds.get(&strike.bolt) else {
        continue;
      };
      let struck: Duration = now.saturating_sub(drive.at.saturating_duration_since(strike.at));

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
      frame.is_burst = drive.burst.is_some();
      frame.listener = listener;
      frame.toggles = drive.toggles;
      emitters.fill(&mut frame.emitters, now);
    });
  }

  /// The sounds the scene would play whose files are missing: its static sounds, its music, its zones', the channels of
  /// the ambient playing, and the rain.
  fn list_missing(&self, weather: Option<&WorldLevelWeather>) -> Vec<WorldSoundRefusal> {
    let mut refused: Vec<WorldSoundRefusal> = Vec::new();
    let mut check = |sound: &WorldSound, kind: WorldSoundKind| {
      if sound.info.is_none() {
        refused.push(WorldSoundRefusal {
          sound: sound.reference.to_string(),
          kind: Some(kind),
          reason: String::from("The sound's file is missing or not a sound the engine loads"),
        });
      }
    };

    if let Some(level) = &self.level {
      for record in level.statics.iter_records() {
        check(&record.sound, WorldSoundKind::Static);
      }

      // Vanilla plays a track's two halves where its stereo file is missing, so the halves are what may be missing then.
      for track in level.music.iter_tracks() {
        match &track.halves {
          Some(halves) if level.engine != XrayEngine::Extended => {
            for half in halves {
              check(half, WorldSoundKind::Music);
            }
          }
          _ => check(&track.stereo, WorldSoundKind::Music),
        }
      }

      for zone in level.zones.iter_records() {
        for sound in [&zone.idle, &zone.disabled].into_iter().flatten() {
          check(sound, zone.get_kind());
        }
      }
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

      for sound in channels {
        check(sound, WorldSoundKind::Channel);
      }

      if let Some(sound) = &weather.rain_sound {
        check(sound, WorldSoundKind::Rain);
      }
    }

    // A missing file is named once, however many sounds name it.
    refused.sort_by(|a, b| a.sound.cmp(&b.sound));
    refused.dedup_by(|a, b| a.sound == b.sound);
    refused
  }
}
