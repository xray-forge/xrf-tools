use glam::Vec3;

use crate::host::render_scene_update::RenderSceneUpdate;
use crate::host::render_streaming_progress::RenderStreamingProgress;

/// What the world gave the frame of the level it shows: what changed in its scene, how far it is streamed, and where
/// its skinned objects' bones stand.
pub struct LevelWorldInput<'a> {
  pub updates: Vec<RenderSceneUpdate>,
  pub streaming: RenderStreamingProgress,
  pub skeleton_segments: &'a [(Vec3, Vec3)],
}
