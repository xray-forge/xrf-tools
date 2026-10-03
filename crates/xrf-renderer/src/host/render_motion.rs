/// A motion baked for a skinned model: every frame's bones in model space, twelve floats a bone as
/// [`crate::RenderModelSkeleton::binds`] holds them, frame after frame.
#[derive(Clone, Debug, Default)]
pub struct RenderMotion {
  pub frames: u32,
  pub bones: u32,
  pub transforms: Vec<f32>,
}
