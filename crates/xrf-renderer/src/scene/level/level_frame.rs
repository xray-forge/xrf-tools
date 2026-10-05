/// What a view's frame was readied with: the readback its pick is copied into, and what its end keeps of what it
/// wrote: the water's reflection while lit, and the resolve whose history it swaps.
pub struct LevelFrame {
  pub pick_slot: Option<usize>,
  pub is_lit: bool,
  pub resolve: Option<&'static str>,
}
