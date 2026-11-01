/// What a view's frame was readied with: the readback its pick is copied into and whether that pick passes the
/// composited surfaces by, and whether it records its scene's own work this frame (the lights' faces), as the first of
/// its scene's views to ready a frame does.
pub struct LevelFrame {
  pub pick_slot: Option<usize>,
  pub is_pick_through: bool,
  pub is_scene_recorder: bool,
}
