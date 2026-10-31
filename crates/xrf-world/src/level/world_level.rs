use std::sync::Arc;
use std::time::Instant;

use glam::Vec3;
use xrf_renderer::{RenderSceneFeedback, RenderSceneUpdate, RenderStreamingProgress, RenderWorkers};

use crate::contract::world_ambient_report::WorldAmbientReport;
use crate::contract::world_level_problems::WorldLevelProblems;
use crate::contract::world_level_reload::WorldLevelReload;
use crate::contract::world_model_pose::WorldModelPose;
use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::host::world_level_source::WorldLevelSource;
use crate::level::animation::level_animation::LevelAnimation;
use crate::level::effects::effects_frame::EffectsFrame;
use crate::level::effects::level_effects::LevelEffects;
use crate::level::level_drive::LevelDrive;
use crate::level::streaming::level_streaming::LevelStreaming;

/// A level as a viewport's world plays it: what streams it into its scene, how its skinned objects are posed, and what
/// its effects play, by one clock running from when it began opening.
pub struct WorldLevel {
  streaming: LevelStreaming,
  animation: LevelAnimation,
  effects: LevelEffects,
  opened: Instant,
  /// What a reload took out of the scene, posted ahead of the next frame's own.
  queued: Vec<RenderSceneUpdate>,
}

impl WorldLevel {
  pub fn start(source: Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      effects: LevelEffects::start(&source, workers),
      streaming: LevelStreaming::start(source, workers),
      animation: LevelAnimation::new(workers),
      opened: Instant::now(),
      queued: Vec::new(),
    }
  }

  pub fn get_source(&self) -> &Arc<dyn WorldLevelSource> {
    self.streaming.get_source()
  }

  /// Plays on from `source`, a next source of the same level, reading again what `reload` names from it: what that
  /// replaces leaves the scene with the next frame's posts, and everything else stays as it is drawn. The clock runs on.
  pub fn reload(&mut self, source: Arc<dyn WorldLevelSource>, reload: WorldLevelReload, workers: &RenderWorkers) {
    if reload.is_particles {
      self.effects.reload(&mut self.queued, &source, workers);
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
  pub fn advance(&mut self, now: Instant, drive: LevelDrive<'_>) -> (Vec<RenderSceneUpdate>, EffectsFrame) {
    let mut updates: Vec<RenderSceneUpdate> = std::mem::take(&mut self.queued);

    self
      .streaming
      .stream(&mut updates, &mut self.animation, drive.toggles.get_spawn_filter());
    self.animation.pose(&mut updates, self.streaming.get_source());

    let (campfires, motions): (&[u16], &[String]) = self.streaming.list_light_followed();
    let effects: EffectsFrame = self.effects.advance(
      &mut updates,
      now.saturating_duration_since(self.opened),
      drive,
      campfires,
      motions,
    );

    (updates, effects)
  }

  pub fn get_progress(&self) -> RenderStreamingProgress {
    RenderStreamingProgress {
      is_particles_done: self.effects.is_read(),
      ..self.streaming.get_progress()
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
