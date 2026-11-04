use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_renderer::{
  LevelPart, LevelParts, RenderSceneFeedback, RenderSceneUpdate, RenderStreamingProgress, RenderWorkers,
};

use crate::contract::level::world_level_problems::WorldLevelProblems;
use crate::contract::level::world_model_pose::WorldModelPose;
use crate::contract::level::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::sound::world_music_command::WorldMusicCommand;
use crate::contract::sound::world_sound_report::WorldSoundReport;
use crate::contract::weather::world_ambient_report::WorldAmbientReport;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::level::animation::level_animation::LevelAnimation;
use crate::level::effects::effects_frame::EffectsFrame;
use crate::level::effects::level_effects::LevelEffects;
use crate::level::level_drive::LevelDrive;
use crate::level::loading::loader_slot::LoaderSlot;
use crate::level::streaming::level_streaming::LevelStreaming;
use crate::sound::level_sound::{EffectSound, LevelSound};
use crate::sound::rain_sound::RainFrame;
use crate::sound::sound_drive::SoundDrive;

/// A level as a viewport's world plays it: what streams it into its scene, how its skinned objects are posed, what its
/// effects play, by one clock running from when it began opening, and what every weather of it plays with, read once
/// for every viewport showing it.
pub struct WorldLevel {
  streaming: LevelStreaming,
  animation: LevelAnimation,
  effects: LevelEffects,
  weather: LoaderSlot<Arc<WorldLevelWeather>>,
  sound: LevelSound,
  opened: Instant,
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
      opened: Instant::now(),
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
  /// spawned models it could not take in of the posts before, and the effects it finished.
  pub fn take_feedback(
    &mut self,
    RenderSceneFeedback {
      sector_failures,
      model_failures,
      finished_effects,
    }: RenderSceneFeedback,
  ) {
    self
      .streaming
      .take_failures(&mut self.animation, sector_failures, model_failures);
    self.effects.note_finished(finished_effects);
  }

  /// This frame's posts to the level's scene at the frame's `now` as its driving viewport gives it, and what its effects
  /// give the scene besides.
  ///
  /// Its sound plays on as `sound` gives the frame: the viewport it is heard by.
  pub fn advance(
    &mut self,
    now: Instant,
    drive: LevelDrive<'_>,
    sound: &SoundDrive<'_>,
  ) -> (Vec<RenderSceneUpdate>, EffectsFrame) {
    let mut updates: Vec<RenderSceneUpdate> = std::mem::take(&mut self.queued);
    let clock: Duration = now.saturating_duration_since(self.opened);

    self
      .streaming
      .stream(&mut updates, &mut self.animation, drive.toggles.get_spawn_filter());
    self.animation.pose(&mut updates, self.streaming.get_source());

    let (campfires, motions): (&[u16], &[String]) = self.streaming.list_light_followed();
    let mut effects: EffectsFrame = self.effects.advance(&mut updates, clock, drive, campfires, motions);
    let weather: Option<Arc<WorldLevelWeather>> = self.weather.poll().cloned();
    let started: Option<(String, Vec3)> = effects.started_effect.take();
    let effect: Option<EffectSound<'_>> = started.as_ref().and_then(|(name, position)| {
      let sound = weather.as_deref()?.ambient_effects.get(name)?.sound.as_ref()?;

      Some(EffectSound {
        effect: name,
        sound,
        position: *position,
      })
    });
    let drive: SoundDrive<'_> = SoundDrive {
      rain: RainFrame {
        hemi: effects.open_sky,
        ..sound.rain
      },
      ..*sound
    };

    self.sound.advance(
      clock,
      self.opened,
      weather.as_deref(),
      &drive,
      effect,
      &mut self.effects,
    );

    (updates, effects)
  }

  /// Does what a viewer asks of the level's music, answering whether it could.
  pub fn command_music(&mut self, command: &WorldMusicCommand) -> bool {
    self.sound.command_music(command)
  }

  /// The level objects whose sounds play at the last frame, by the ids their helpers carry, ascending.
  pub fn list_sounding(&self) -> Vec<u32> {
    self.sound.list_sounding()
  }

  /// What its sound plays as the world schedules it.
  pub fn report_sound(&mut self) -> WorldSoundReport {
    let weather: Option<Arc<WorldLevelWeather>> = self.weather.poll().cloned();

    self.sound.report(weather.as_deref())
  }

  pub fn get_progress(&self) -> RenderStreamingProgress {
    let streamed: RenderStreamingProgress = self.streaming.get_progress();

    RenderStreamingProgress {
      done: streamed.done.with_if(LevelPart::Particles, self.effects.is_read()),
      ..streamed
    }
  }

  /// Every skinned object's bones as segments in renderer space, child then parent, where this frame poses them.
  pub fn list_segments(&self) -> Arc<[(Vec3, Vec3)]> {
    self.animation.list_segments()
  }

  pub fn measure_surfaces(&self) -> Vec<WorldSurfaceGeometry> {
    self.streaming.measure_surfaces()
  }

  pub fn describe_problems(&self) -> WorldLevelProblems {
    self.streaming.describe_problems()
  }
}
