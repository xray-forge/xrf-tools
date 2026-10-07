use std::collections::HashMap;

use glam::{Mat4, Vec3};

use crate::host::render_scene_update::RenderSceneUpdate;
use crate::host::render_streaming_progress::RenderStreamingProgress;
use crate::lighting::ambient_gust::AmbientGust;

/// What the world gave the frame of the level it shows: what changed in its scene, how far it is streamed, where its
/// skinned objects' bones stand, and what its effects and moving zones do.
pub struct LevelWorldInput<'a> {
  pub updates: Vec<RenderSceneUpdate>,
  pub streaming: RenderStreamingProgress,
  pub skeleton_segments: &'a [(Vec3, Vec3)],
  /// The wind the ambient effects blow, the campfires' idle lights' shares, and where the object motions have their
  /// objects.
  pub gust: AmbientGust,
  pub campfire_shares: &'a HashMap<u16, f32>,
  pub motions: &'a HashMap<String, (Mat4, Vec3)>,
}
