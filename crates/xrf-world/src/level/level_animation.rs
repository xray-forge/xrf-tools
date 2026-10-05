use std::collections::HashMap;
use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_renderer::{
  RenderLevelSource, RenderModelSkeleton, RenderMotion, RenderSceneUpdate, RenderWorkers, StaticObjectProxy,
};
use xrf_renderer_core::ProxyHandle;

use crate::contract::world_model_pose::WorldModelPose;
use crate::level::model_motions::ModelMotions;
use crate::level::posed_skeleton::PosedSkeleton;

/// The skinned objects standing in a level's scene and how they are posed: each one's skeleton and place, the motions
/// they are posed by, and the pose asked for, every object posting its bone matrices each frame.
pub struct LevelAnimation {
  skeletons: HashMap<ProxyHandle<StaticObjectProxy>, (PosedSkeleton, Mat4)>,
  motions: ModelMotions,
  pose: WorldModelPose,
}

impl LevelAnimation {
  pub fn new(workers: &RenderWorkers) -> Self {
    Self {
      skeletons: HashMap::new(),
      motions: ModelMotions::new(workers),
      pose: WorldModelPose::default(),
    }
  }

  /// Stands every skinned object as asked from the next frame on.
  pub fn set_pose(&mut self, pose: &WorldModelPose) {
    if self.pose != *pose {
      self.pose = pose.clone();
    }
  }

  /// Poses an object newly standing in the scene, where `transform` stands it.
  pub fn add(&mut self, object: ProxyHandle<StaticObjectProxy>, skeleton: &RenderModelSkeleton, transform: Mat4) {
    self.skeletons.insert(object, (PosedSkeleton::new(skeleton), transform));
  }

  pub fn remove(&mut self, object: ProxyHandle<StaticObjectProxy>) {
    self.skeletons.remove(&object);
  }

  /// Posts every skinned object's bone matrices for this frame, and the last frame's beside them; a motion still on its
  /// way poses the bind pose meanwhile.
  pub fn pose(&mut self, updates: &mut Vec<RenderSceneUpdate>, source: &Arc<dyn RenderLevelSource>) {
    if self.skeletons.is_empty() {
      return;
    }

    let pose: &WorldModelPose = &self.pose;
    let motion: Option<&RenderMotion> = match &pose.motion {
      Some(name) => self.motions.get(source, name),
      None => None,
    };

    for (object, (skeleton, _)) in &mut self.skeletons {
      let (current, previous) = skeleton.pose(motion, pose.frame, &pose.hidden_bones);

      updates.push(RenderSceneUpdate::PoseObject {
        handle: *object,
        current,
        previous,
      });
    }
  }

  /// Every skinned object's bones as segments in renderer space, child then parent, where this frame poses them.
  pub fn list_segments(&self) -> Vec<(Vec3, Vec3)> {
    self
      .skeletons
      .values()
      .flat_map(|(skeleton, place)| {
        skeleton
          .list_segments()
          .into_iter()
          .map(move |(child, parent)| (place.transform_point3(child), place.transform_point3(parent)))
      })
      .collect()
  }
}
