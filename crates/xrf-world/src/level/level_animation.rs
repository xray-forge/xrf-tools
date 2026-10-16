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
/// they are posed by, and the pose asked for, an object posting its bone matrices only until it stands as asked.
pub struct LevelAnimation {
  skeletons: HashMap<ProxyHandle<StaticObjectProxy>, (PosedSkeleton, Mat4)>,
  motions: ModelMotions,
  pose: WorldModelPose,
  /// The pose every object was last posed by, and whether its motion had arrived then; none before the first.
  posed: Option<(WorldModelPose, bool)>,
  /// Every object's bones as segments in renderer space, made again only when one moves, comes or goes.
  segments: Arc<[(Vec3, Vec3)]>,
  is_segments_stale: bool,
}

impl LevelAnimation {
  pub fn new(workers: &RenderWorkers) -> Self {
    Self {
      skeletons: HashMap::new(),
      motions: ModelMotions::new(workers),
      pose: WorldModelPose::default(),
      posed: None,
      segments: Arc::from([]),
      is_segments_stale: false,
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
    self.is_segments_stale |= self.skeletons.remove(&object).is_some();
  }

  /// Posts the bone matrices of each skinned object whose pose changes this frame, and the last frame's beside them:
  /// every object where the pose asked for or its motion's arrival changed, else one new or still settling, until it
  /// stands as asked. A motion still on its way poses the bind pose meanwhile.
  pub fn pose(&mut self, updates: &mut Vec<RenderSceneUpdate>, source: &Arc<dyn RenderLevelSource>) {
    let pose: &WorldModelPose = &self.pose;
    let motion: Option<&RenderMotion> = match &pose.motion {
      Some(name) if !self.skeletons.is_empty() => self.motions.get(source, name),
      _ => None,
    };
    let is_arrived: bool = motion.is_some();
    let is_changed: bool = self
      .posed
      .as_ref()
      .is_none_or(|(posed, was_arrived)| posed != pose || *was_arrived != is_arrived);
    let mut is_moved: bool = false;

    for (object, (skeleton, _)) in &mut self.skeletons {
      if !is_changed && skeleton.is_settled() {
        continue;
      }

      let (current, previous) = skeleton.pose(motion, pose.frame, &pose.hidden_bones);

      updates.push(RenderSceneUpdate::PoseObject {
        handle: *object,
        current,
        previous,
      });
      is_moved = true;
    }

    if is_changed {
      self.posed = Some((pose.clone(), is_arrived));
    }

    if is_moved || self.is_segments_stale {
      self.segments = self.collect_segments();
      self.is_segments_stale = false;
    }
  }

  /// Every skinned object's bones as segments in renderer space, child then parent, where this frame poses them.
  pub fn list_segments(&self) -> Arc<[(Vec3, Vec3)]> {
    Arc::clone(&self.segments)
  }

  fn collect_segments(&self) -> Arc<[(Vec3, Vec3)]> {
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
