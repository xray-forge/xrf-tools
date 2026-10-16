use std::sync::Arc;
use std::time::Instant;

use glam::Vec3;
use xrf_renderer::{
  ParticleEmitterProxy, PlacedEffect, RenderLevelSource, RenderModelFailure, RenderSceneUpdate, RenderSectorFailure,
  RenderStreamingProgress, RenderViewOptions, RenderWorkers,
};
use xrf_renderer_core::ProxyHandle;

use crate::contract::world_ambient_report::WorldAmbientReport;
use crate::contract::world_level_problems::WorldLevelProblems;
use crate::contract::world_model_pose::WorldModelPose;
use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::world_toggles::WorldToggles;
use crate::level::ambient_frame::AmbientFrame;
use crate::level::effects_frame::EffectsFrame;
use crate::level::level_animation::LevelAnimation;
use crate::level::level_effects::LevelEffects;
use crate::level::level_streaming::LevelStreaming;

/// A level as a viewport's world plays it: what streams it into its scene, how its skinned objects are posed, and what
/// its effects play, by one clock running from when it began opening.
pub struct WorldLevel {
  streaming: LevelStreaming,
  animation: LevelAnimation,
  effects: LevelEffects,
  opened: Instant,
}

impl WorldLevel {
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      effects: LevelEffects::start(&source, workers),
      streaming: LevelStreaming::start(source, workers),
      animation: LevelAnimation::new(workers),
      opened: Instant::now(),
    }
  }

  pub fn get_source(&self) -> &Arc<dyn RenderLevelSource> {
    self.streaming.get_source()
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

  /// Takes what the level's scene told of the last frame, whether or not a viewport drives it now: `failures` are the
  /// sectors and the spawned models it could not take in of the posts before, and `finished` the effects it finished.
  pub fn take_feedback(
    &mut self,
    failures: (Vec<RenderSectorFailure>, Vec<RenderModelFailure>),
    finished: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
  ) {
    self.streaming.take_failures(&mut self.animation, failures);
    self.effects.note_finished(finished);
  }

  /// This frame's posts to the level's scene at the frame's `now` by the view's options and the frame's weather, the
  /// camera standing at `eye` in engine space, and what its effects give the scene besides.
  pub fn advance(
    &mut self,
    now: Instant,
    (options, toggles, ambient): (&RenderViewOptions, &WorldToggles, Option<AmbientFrame<'_>>),
    eye: Vec3,
  ) -> (Vec<RenderSceneUpdate>, EffectsFrame) {
    let mut updates: Vec<RenderSceneUpdate> = Vec::new();

    self
      .streaming
      .stream(&mut updates, &mut self.animation, toggles.get_hidden_spawn_groups());
    self.animation.pose(&mut updates, self.streaming.get_source());

    let effects: EffectsFrame = self.effects.advance(
      &mut updates,
      now.saturating_duration_since(self.opened),
      (options, toggles, ambient),
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
