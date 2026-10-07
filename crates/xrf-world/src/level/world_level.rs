use std::sync::Arc;

use glam::Vec3;
use xrf_renderer::{
  RenderLevelProblems, RenderLevelSource, RenderModelPose, RenderSceneUpdate, RenderSectorFailure,
  RenderStreamingProgress, RenderSurfaceGeometry, RenderWorkers,
};

use crate::level::level_animation::LevelAnimation;
use crate::level::level_streaming::LevelStreaming;

/// A level as a viewport's world plays it: what streams it into its scene, and how its skinned objects are posed.
pub struct WorldLevel {
  streaming: LevelStreaming,
  animation: LevelAnimation,
}

impl WorldLevel {
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
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

  /// This frame's posts to the level's scene, the spawn groups `hidden` names left out; `failures` are what the scene
  /// could not take in of the last frame's.
  pub fn advance(&mut self, hidden: u32, failures: Vec<RenderSectorFailure>) -> Vec<RenderSceneUpdate> {
    let mut updates: Vec<RenderSceneUpdate> = Vec::new();

    self
      .streaming
      .stream(&mut updates, &mut self.animation, (hidden, failures));
    self.animation.pose(&mut updates, self.streaming.get_source());

    updates
  }

  pub fn get_progress(&self) -> RenderStreamingProgress {
    self.streaming.get_progress()
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
