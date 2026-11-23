use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_math::glam::Vec3;

use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::host::level::render_level_source::RenderLevelSource;
use crate::host::world::feedback::render_streaming_progress::RenderStreamingProgress;
use crate::host::world::render_motion_pose::RenderMotionPose;
use crate::host::world::render_scene_update::RenderSceneUpdate;
use crate::lighting::wind::ambient_gust::AmbientGust;

/// What the world gives a scene each frame: its source, clocks, updates, streaming progress, driving viewport, and its
/// effects' and moving zones' state.
pub struct RenderSceneFrame {
  pub source: Arc<dyn RenderLevelSource>,
  /// The frame on which the world first played the level; the level's clock (`LevelScene::get_time`), effects and
  /// sound all run from it, so they keep one time.
  pub opened: Instant,
  /// The engine's time (`Device.dwTimeGlobal`): frame steps summed, never running back, which particles step by as
  /// the world's effects do, so a burst's jump of the level clock is one short step to them.
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
  /// Whether the lights of the artefacts a detector must reveal are lit, as their models are drawn.
  pub is_detector_hidden_lit: bool,
  /// Each object motion's pose and velocity this frame, by motion name.
  pub motions: HashMap<String, RenderMotionPose>,
  /// The driving camera's smoothed hemi, which says how far outdoors it stands; none where it is not estimated.
  pub camera_hemi: Option<f32>,
  /// Ids, ascending, of the level objects whose sounds play this frame; their helpers draw lit.
  pub sounding: Vec<u32>,
}
