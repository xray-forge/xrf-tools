use std::sync::Arc;

use glam::Vec3;
use xrf_renderer::{
  ParticleEmitterProxy, PlacedEffect, RenderAmbientReport, RenderLevelProblems, RenderLevelSource, RenderModelPose,
  RenderSceneUpdate, RenderSectorFailure, RenderStreamingProgress, RenderSurfaceGeometry, RenderViewOptions,
  RenderWorkers,
};
use xrf_renderer_core::ProxyHandle;

use crate::level::ambient_frame::AmbientFrame;
use crate::level::effects_frame::EffectsFrame;
use crate::level::level_animation::LevelAnimation;
use crate::level::level_effects::LevelEffects;
use crate::level::level_streaming::LevelStreaming;

/// A level as a viewport's world plays it: what streams it into its scene, how its skinned objects are posed, and what
/// its effects play.
pub struct WorldLevel {
  streaming: LevelStreaming,
  animation: LevelAnimation,
  effects: LevelEffects,
}

impl WorldLevel {
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      effects: LevelEffects::start(&source, workers),
      streaming: LevelStreaming::start(source, workers),
      animation: LevelAnimation::new(workers),
    }
  }

  pub fn get_source(&self) -> &Arc<dyn RenderLevelSource> {
    self.streaming.get_source()
  }

  pub fn set_pose(&mut self, pose: &RenderModelPose) {
    self.animation.set_pose(pose);
  }

  /// Plays an ambient effect on the next frame, ending the one playing, without waiting.
  pub fn play_ambient_now(&mut self) {
    self.effects.play_ambient_now();
  }

  /// Where the weather's ambient effects stand, none until the particles are read.
  pub fn report_ambient(&self) -> Option<RenderAmbientReport> {
    self.effects.report_ambient()
  }

  /// This frame's posts to the level's scene by the view's options and the frame's weather, the camera standing at
  /// `eye` in engine space, and what its effects give the scene besides. `failures` are the sectors the scene could not
  /// take in of the last frame's posts, and `finished` the effects it finished.
  pub fn advance(
    &mut self,
    (options, ambient): (&RenderViewOptions, Option<AmbientFrame<'_>>),
    eye: Vec3,
    failures: Vec<RenderSectorFailure>,
    finished: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
  ) -> (Vec<RenderSceneUpdate>, EffectsFrame) {
    let mut updates: Vec<RenderSceneUpdate> = Vec::new();

    self.streaming.stream(
      &mut updates,
      &mut self.animation,
      (options.world.get_hidden_spawn_groups(), failures),
    );
    self.animation.pose(&mut updates, self.streaming.get_source());

    self.effects.note_finished(finished);

    let effects: EffectsFrame = self.effects.advance(
      &mut updates,
      (options, ambient),
      eye,
      self.streaming.list_light_followed(),
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
  pub fn list_segments(&self) -> Vec<(Vec3, Vec3)> {
    self.animation.list_segments()
  }

  pub fn measure_surfaces(&self) -> Vec<RenderSurfaceGeometry> {
    self.streaming.measure_surfaces()
  }

  pub fn describe_problems(&self) -> RenderLevelProblems {
    self.streaming.describe_problems()
  }
}
