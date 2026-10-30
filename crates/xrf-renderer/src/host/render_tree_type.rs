/// One of a level's tree types as its baked impostor needs it: how many trees the level stands of it, which decides
/// the size it bakes at, and a key of what draws it, which its bake is cached by across levels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderTreeType {
  pub trees: u32,
  pub key: u64,
}
