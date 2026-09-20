use crate::pack::visual::motion::visual_motion_bake::VisualMotionBake;

/// Bone transforms of one baked motion, frame major: frame 0's bones, then frame 1's.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualMotionPose {
  pub description: VisualMotionBake,
  pub transforms: Vec<f32>,
}
