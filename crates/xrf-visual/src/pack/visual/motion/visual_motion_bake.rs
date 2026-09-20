use serde::Serialize;

/// What one baked motion is, beside the frames themselves.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualMotionBake {
  pub name: String,
  /// Frames the buffer holds: the longest key stream the payload carries, not the count the motion declares.
  pub frame_count: u32,
  pub bone_count: u32,
  /// Seconds playing the motion takes: its frames at the format's sample rate, over its playback speed.
  pub duration: f32,
  /// The playback speed the motion's definition declares, as stored.
  pub speed: f32,
  /// How many bones the motion actually drives, the rest holding their bind pose.
  pub animated_bone_count: u32,
  /// Floats one bone's transform occupies in the baked buffer, so a consumer indexes it without agreeing a constant.
  pub floats_per_bone: u32,
}
