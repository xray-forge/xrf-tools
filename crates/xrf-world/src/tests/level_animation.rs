use std::sync::Arc;

use glam::Mat4;
use xrf_renderer::{RenderLevelSource, RenderModelSkeleton, RenderSceneUpdate, StaticObjectProxy};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::contract::world_model_pose::WorldModelPose;
use crate::level::animation::level_animation::LevelAnimation;
use crate::tests::fixtures::TestLevelSource;
use crate::tests::test_workers::create_workers;

/// How many objects one frame of `animation` posts poses for.
fn count_posed(animation: &mut LevelAnimation, source: &Arc<dyn RenderLevelSource>) -> usize {
  let mut updates: Vec<RenderSceneUpdate> = Vec::new();

  animation.pose(&mut updates, source);
  updates
    .iter()
    .filter(|update| matches!(update, RenderSceneUpdate::PoseObject { .. }))
    .count()
}

// A pose is posted until it settles, the frame after a change carrying the last frame's matrices beside it, then not
// again until another is asked.
#[test]
fn posts_a_pose_only_until_it_settles() {
  let source: Arc<dyn RenderLevelSource> = Arc::new(TestLevelSource::default());
  let mut animation: LevelAnimation = LevelAnimation::new(&create_workers());
  let mut handles: ProxyAllocator<StaticObjectProxy> = ProxyAllocator::new();
  let skeleton: RenderModelSkeleton = RenderModelSkeleton {
    binds: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
    pairs: vec![(0, 0)],
  };
  let object: ProxyHandle<StaticObjectProxy> = handles.allocate();

  animation.add(object, &skeleton, Mat4::IDENTITY);

  assert_eq!(count_posed(&mut animation, &source), 1, "a new object is posed");
  assert_eq!(count_posed(&mut animation, &source), 0, "its first pose drew as itself");

  animation.set_pose(&WorldModelPose {
    hidden_bones: vec![0],
    ..WorldModelPose::default()
  });

  assert_eq!(count_posed(&mut animation, &source), 1, "a new pose is posted");
  assert_eq!(
    count_posed(&mut animation, &source),
    1,
    "and once more, as the last frame's"
  );
  assert_eq!(count_posed(&mut animation, &source), 0, "then it stands as asked");
}
