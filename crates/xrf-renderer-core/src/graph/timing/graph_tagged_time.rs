/// What one pass, or one render pass of merged passes, cost on the GPU in one frame, by the tag that frame was timed
/// under: milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphTaggedTime {
  pub tag: u64,
  pub name: String,
  pub gpu_time: f32,
}
