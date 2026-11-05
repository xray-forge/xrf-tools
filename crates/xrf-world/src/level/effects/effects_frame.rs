use std::collections::HashMap;

use xrf_renderer::{AmbientGust, RenderMotionPose};

use crate::level::ambient::started_effect::StartedEffect;

/// What a frame of a level's effects gives its scene besides its posts: the wind the ambient effects blow, how much of
/// each campfire's idle light shows by its id, and where each object motion has its object in engine space and how fast
/// it moves, by the motion's name.
#[derive(Default)]
pub struct EffectsFrame {
  pub gust: AmbientGust,
  pub campfire_shares: HashMap<u16, f32>,
  pub motions: HashMap<String, RenderMotionPose>,
  /// The camera's smoothed hemi, none where the level has no estimator.
  pub camera_hemi: Option<f32>,
  /// How open to the sky the camera stands for the rain's loop, none where the level has no estimator.
  pub open_sky: Option<f32>,
  /// The ambient effect started this frame.
  pub started_effect: Option<StartedEffect>,
}
