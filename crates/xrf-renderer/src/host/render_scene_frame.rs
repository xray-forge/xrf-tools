use std::collections::HashMap;
use std::sync::Arc;

use glam::{Mat4, Vec3};

use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_scene_update::RenderSceneUpdate;
use crate::host::render_streaming_progress::RenderStreamingProgress;
use crate::lighting::ambient_gust::AmbientGust;

/// What the world gives a scene's frame: the level it is, what changed in it, how far it is streamed, which viewport
/// drives what it simulates by its camera, and what its effects and moving zones do.
pub struct RenderSceneFrame {
  pub source: Arc<dyn RenderLevelSource>,
  /// The viewport whose camera stands for the actor: the first still showing the scene.
  pub driver: Option<RenderViewportId>,
  pub updates: Vec<RenderSceneUpdate>,
  pub streaming: RenderStreamingProgress,
  /// Every skinned object's bones as segments in renderer space, child then parent, which the skeleton overlay draws.
  pub skeleton_segments: Vec<(Vec3, Vec3)>,
  /// The wind the ambient effects blow this frame, which the grass, the rain and the campfires read.
  pub gust: AmbientGust,
  /// How much of each campfire's idle light shows this frame, by its spawned object's id.
  pub campfire_shares: HashMap<u16, f32>,
  /// Where each object motion has its object this frame, in engine space, and how fast it moves, by the motion's name.
  pub motions: HashMap<String, (Mat4, Vec3)>,
  /// The driving camera's smoothed hemi, which says how far outdoors it stands; none where it is not estimated.
  pub camera_hemi: Option<f32>,
}
