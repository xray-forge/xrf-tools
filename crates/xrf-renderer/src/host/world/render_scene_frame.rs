use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec3;

use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::host::level::render_level_source::RenderLevelSource;
use crate::host::world::feedback::render_streaming_progress::RenderStreamingProgress;
use crate::host::world::render_motion_pose::RenderMotionPose;
use crate::host::world::render_scene_update::RenderSceneUpdate;
use crate::lighting::wind::ambient_gust::AmbientGust;

/// What the world gives a scene's frame: the level it is, when its clock began and the engine's time, what changed in
/// it, how far it is streamed, which viewport drives what it simulates by its camera, and what its effects and moving
/// zones do.
pub struct RenderSceneFrame {
  pub source: Arc<dyn RenderLevelSource>,
  /// When the level's clock began on the frames' clock, which the world plays the level's effects and sound from: the
  /// first frame it played the level on. The scene's time runs from it too (`LevelScene::get_time`), so the picture, the
  /// effects and the sound keep one time.
  pub opened: Instant,
  /// The engine's time at this frame (the world's engine clock, `Device.dwTimeGlobal`): the level's clock added up a
  /// frame at a time, never back, which the scene's particles step and start by, as the world's effects do, so a
  /// burst's jump of the level's clock is one short step to them.
  pub engine_time: Duration,
  /// The viewport whose camera stands for the actor: the first still showing the scene.
  pub driver: Option<RenderViewportId>,
  pub updates: Vec<RenderSceneUpdate>,
  pub streaming: RenderStreamingProgress,
  /// Every skinned object's bones as segments in renderer space, child then parent, which the skeleton overlay draws.
  pub skeleton_segments: Arc<[(Vec3, Vec3)]>,
  /// The wind the ambient effects blow this frame, which the grass, the rain and the campfires read.
  pub gust: AmbientGust,
  /// How much of each campfire's idle light shows this frame, by its spawned object's id.
  pub campfire_shares: HashMap<u16, f32>,
  /// Where each object motion has its object this frame, and how fast it moves, by the motion's name.
  pub motions: HashMap<String, RenderMotionPose>,
  /// The driving camera's smoothed hemi, which says how far outdoors it stands; none where it is not estimated.
  pub camera_hemi: Option<f32>,
  /// The helpers of the level objects whose sounds play this frame, by the ids they carry, ascending, which every view
  /// draws lit.
  pub sounding: Vec<u32>,
}
