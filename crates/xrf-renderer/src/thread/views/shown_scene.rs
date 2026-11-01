use crate::scene::level::level_scene::LevelScene;

/// The scene a view shows, and whether the view is its first this frame, which draws the scene's own work.
pub struct ShownScene<'s> {
  pub scene: &'s mut LevelScene,
  pub is_first: bool,
}
