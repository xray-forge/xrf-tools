/// What a view's frame was readied with: the readback its pick is copied into, whether it is the first of its scene's
/// views, which declares the scene's own work, and what its end keeps of what it wrote: the water's reflection while
/// lit, and the resolve whose history it swaps.
pub struct LevelFrame {
  pub pick_slot: Option<usize>,
  pub is_scene_first: bool,
  pub is_lit: bool,
  pub resolve: Option<&'static str>,
}
