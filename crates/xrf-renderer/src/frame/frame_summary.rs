use crate::contract::render_frame_phases::RenderFramePhases;
use crate::contract::render_group_cost::RenderGroupCost;

/// What a span of frames cost, in milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameSummary {
  pub frames_per_second: f32,
  pub frame_time: f32,
  pub frame_time_max: f32,
  pub frame_times: Vec<f32>,
  pub cpu_time: f32,
  pub cpu_times: Vec<f32>,
  pub gpu_time: Option<f32>,
  pub gpu_times: Vec<f32>,
  pub phases: RenderFramePhases,
  pub groups: Vec<RenderGroupCost>,
}
