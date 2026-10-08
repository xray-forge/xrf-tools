/// What a view's frame was readied with: the readback its pick is copied into and whether that pick passes the
/// composited surfaces by, whether it is the first of its scene's
/// views, which declares the scene's own work, and whether it is lit, which decides what its end keeps of what it
/// wrote.
pub struct LevelFrame {
  pub pick_slot: Option<usize>,
  pub is_pick_through: bool,
  pub is_scene_first: bool,
  pub is_lit: bool,
}
