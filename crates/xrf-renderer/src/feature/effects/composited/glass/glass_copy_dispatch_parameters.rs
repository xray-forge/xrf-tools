use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, StorageArrayMut};

/// What the glass copy's dispatch reads and writes: the cull's early and late draw arguments, whose glass lists say
/// whether any glass draws this frame, the frame whose size the copy covers, and the dispatch it writes for the copy.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct GlassCopyDispatchParameters {
  #[storage]
  pub early: StorageArray<u32>,
  #[storage]
  pub late: StorageArray<u32>,
  #[texture(d2, float)]
  pub scene: GraphTexture,
  #[storage]
  pub dispatch: StorageArrayMut<u32>,
}
