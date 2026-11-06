use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer::{
  LevelPart, LevelParts, RenderLevelSource, RenderSceneFeedback, RenderSceneFrame, RenderSceneUpdate,
  RenderStreamingProgress, RenderViewBurst, RenderViewportId, RenderWorkers,
};

use crate::contract::level::world_level_problems::WorldLevelProblems;
use crate::contract::level::world_model_pose::WorldModelPose;
use crate::contract::level::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::sound::report::world_sound_report::WorldSoundReport;
use crate::contract::sound::world_music_command::WorldMusicCommand;
use crate::contract::weather::world_ambient_report::WorldAmbientReport;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::level::ambient::started_effect::StartedEffect;
use crate::level::animation::level_animation::LevelAnimation;
use crate::level::effects::effect_clocks::EffectClocks;
use crate::level::effects::effects_frame::EffectsFrame;
use crate::level::effects::level_effects::LevelEffects;
use crate::level::engine_clock::EngineClock;
use crate::level::level_drive::LevelDrive;
use crate::level::loading::loader_slot::LoaderSlot;
use crate::level::streaming::level_streaming::LevelStreaming;
use crate::sound::level::effect_sound::EffectSound;
use crate::sound::level::level_sound::LevelSound;
use crate::sound::level::sound_drive::SoundDrive;
use crate::sound::weather::rain_frame::RainFrame;

/// A level as a viewport's world plays it: what streams it into its scene, how its skinned objects are posed, what its
/// effects play, and what every weather of it plays with, read once for every viewport showing it.
///
/// It owns the level's clock: begun on the frames' clock at the first frame it plays, which its scene is made on and
/// told of (`RenderSceneFrame::opened`), so its effects, its sound and its picture keep one time; and the engine's clock
/// its effects and its scene's particles step by (`EngineClock`, `RenderSceneFrame::engine_time`). Its sound waits for
/// the scene to stand ready, then starts every schedule at that frame, as the game starts a level's sound once it has
/// loaded; a burst is heard whether or not it stands ready, as it starts its schedules afresh itself.
pub struct WorldLevel {
  streaming: LevelStreaming,
  animation: LevelAnimation,
  effects: LevelEffects,
  weather: LoaderSlot<Arc<WorldLevelWeather>>,
  sound: LevelSound,
  /// When the level's clock began, on the frames' clock; none before its first frame.
  opened: Option<Instant>,
  /// The engine's clock its effects and its scene's particles step by, which never jumps as the level's clock may.
  engine_clock: EngineClock,
  /// Whether the level's scene has stood ready, as it told the world.
  has_stood_ready: bool,
  /// What a reload took out of the scene, posted ahead of the next frame's own.
  queued: Vec<RenderSceneUpdate>,
}

impl WorldLevel {
  pub fn start(source: Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      effects: LevelEffects::start(&source, workers),
      weather: Self::read_weather(&source, workers),
      sound: LevelSound::start(&source, workers),
      streaming: LevelStreaming::start(source, workers),
      animation: LevelAnimation::new(workers),
      opened: None,
      engine_clock: EngineClock::default(),
      has_stood_ready: false,
      queued: Vec::new(),
    }
  }

  /// Reads what every weather of the level plays with on the workers.
  fn read_weather(source: &Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> LoaderSlot<Arc<WorldLevelWeather>> {
    let source: Arc<dyn WorldLevelSource> = Arc::clone(source);

    LoaderSlot::spawn(workers, "weather", move || {
      source
        .read_weather()
        .map(Arc::new)
        .inspect_err(|error| log::warn!("The level's weather cannot be played: {error}"))
        .ok()
    })
  }

  pub fn get_source(&self) -> &Arc<dyn WorldLevelSource> {
    self.streaming.get_source()
  }

  /// What every weather of the level plays with, once read; none while it is read, again on a reload too, or where it
  /// cannot be.
  pub fn get_weather(&mut self) -> Option<&Arc<WorldLevelWeather>> {
    self.weather.poll()
  }

  /// Plays on from `source`, a next source of the same level, reading again what `reload` names from it: what that
  /// replaces leaves the scene with the next frame's posts, and everything else stays as it is drawn. The clock runs on.
  pub fn reload(&mut self, source: Arc<dyn WorldLevelSource>, reload: LevelParts, workers: &RenderWorkers) {
    if reload.contains(LevelPart::Particles) {
      self.effects.reload(&mut self.queued, &source, workers);
    }

    if reload.contains(LevelPart::Weather) {
      self.weather = Self::read_weather(&source, workers);
    }

    self.effects.follow_motions(&source, reload);

    // The level's sounds are read from what its helpers are, its level objects, and each names its helper by its place
    // among them.
    if reload.contains(LevelPart::Helpers) {
      self.sound.reload(&source, workers);
    }

    self
      .streaming
      .follow(&mut self.queued, &mut self.animation, source, reload, workers);
  }

  pub fn set_pose(&mut self, pose: &WorldModelPose) {
    self.animation.set_pose(pose);
  }

  /// Plays an ambient effect on the next frame, ending the one playing, without waiting.
  pub fn play_ambient_now(&mut self) {
    self.effects.play_ambient_now();
  }

  /// Where the weather's ambient effects stand, none until the particles are read.
  pub fn report_ambient(&self) -> Option<WorldAmbientReport> {
    self.effects.report_ambient()
  }

  /// Takes what the level's scene told of the last frame, whether or not a viewport drives it now: the sectors and the
  /// spawned models it could not take in of the posts before, the effects it finished, and whether it stood ready.
  pub fn take_feedback(
    &mut self,
    RenderSceneFeedback {
      sector_failures,
      model_failures,
      finished_effects,
      has_stood_ready,
    }: RenderSceneFeedback,
  ) {
    self
      .streaming
      .take_failures(&mut self.animation, sector_failures, model_failures);
    self.effects.note_finished(finished_effects);
    self.has_stood_ready |= has_stood_ready;
  }

  /// The level's scene's frame at the frame's `now`, on the frames' clock, as its driving viewport gives it: what it
  /// posts, what its effects give the scene besides, and when its clock began.
  ///
  /// Its sound plays on as `sound` gives the frame, the viewport it is heard by, once the scene stands ready or a burst
  /// draws it.
  pub fn advance(&mut self, now: Instant, drive: LevelDrive<'_>, sound: &SoundDrive<'_>) -> RenderSceneFrame {
    let mut updates: Vec<RenderSceneUpdate> = std::mem::take(&mut self.queued);
    let opened: Instant = *self.opened.get_or_insert(now);
    let clock: Duration = now.saturating_duration_since(opened);
    let engine: Duration = self.engine_clock.advance(clock);
    let driver: RenderViewportId = drive.viewport;

    self
      .streaming
      .stream(&mut updates, &mut self.animation, drive.toggles.get_spawn_filter());
    self.animation.pose(&mut updates, self.streaming.get_source());

    // A burst's first frame starts the ambient effects afresh, as it starts the sound's schedules.
    if sound.burst == Some(RenderViewBurst::Starting) {
      self.effects.restart_ambient(&mut updates);
    }

    let mut effects: EffectsFrame = self.effects.advance(
      &mut updates,
      EffectClocks { level: clock, engine },
      drive,
      self.streaming.list_light_campfires(),
      self.streaming.list_light_motions(),
    );
    let weather: Option<Arc<WorldLevelWeather>> = self.weather.poll().cloned();
    let started: Option<StartedEffect> = effects.started_effect.take();
    let effect: Option<EffectSound<'_>> = started.as_ref().and_then(|started| {
      let sound = weather.as_deref()?.ambient_effects.get(&started.name)?.sound.as_ref()?;

      Some(EffectSound {
        effect: &started.name,
        sound,
        position: started.position,
      })
    });
    let drive: SoundDrive<'_> = SoundDrive {
      rain: RainFrame {
        hemi: effects.open_sky,
        ..sound.rain
      },
      ..*sound
    };

    if self.has_stood_ready || drive.burst.is_some() {
      self
        .sound
        .advance(clock, weather.as_deref(), &drive, effect, &mut self.effects);
    }

    RenderSceneFrame {
      source: Arc::clone(self.streaming.get_source()) as Arc<dyn RenderLevelSource>,
      opened,
      engine_time: engine,
      driver: Some(driver),
      updates,
      streaming: self.get_progress(),
      skeleton_segments: self.animation.list_segments(),
      gust: effects.gust,
      campfire_shares: effects.campfire_shares,
      motions: effects.motions,
      camera_hemi: effects.camera_hemi,
      // The level objects whose sounds play at this frame, by the ids their helpers carry, ascending.
      sounding: self.sound.list_sounding(),
    }
  }

  /// Does what a viewer asks of the level's music, answering whether it could.
  pub fn command_music(&mut self, command: &WorldMusicCommand) -> bool {
    self.sound.command_music(command)
  }

  /// What its sound plays as the world schedules it.
  pub fn report_sound(&mut self) -> WorldSoundReport {
    let weather: Option<Arc<WorldLevelWeather>> = self.weather.poll().cloned();

    self.sound.report(weather.as_deref())
  }

  /// How far the level is streamed into its scene, its particles read among its parts.
  fn get_progress(&self) -> RenderStreamingProgress {
    let streamed: RenderStreamingProgress = self.streaming.get_progress();

    RenderStreamingProgress {
      done: streamed.done.with_if(LevelPart::Particles, self.effects.is_read()),
      ..streamed
    }
  }

  pub fn measure_surfaces(&self) -> Vec<WorldSurfaceGeometry> {
    self.streaming.measure_surfaces()
  }

  pub fn describe_problems(&self) -> WorldLevelProblems {
    self.streaming.describe_problems()
  }
}
