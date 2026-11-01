/// A skinned model's skeleton as the renderer poses it: each bone's bind in model space, twelve floats a bone (its
/// basis' three columns, then its translation), and the bones the overlay joins, child then parent.
#[derive(Clone, Debug, Default)]
pub struct RenderModelSkeleton {
  pub binds: Vec<f32>,
  pub pairs: Vec<(u16, u16)>,
}
