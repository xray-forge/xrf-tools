use serde::Serialize;

/// What one pass, or one render pass of merged passes, cost on the GPU: mean milliseconds over the frames timed.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphPassTime {
  pub name: String,
  pub gpu_time: f32,
}
