/// How an overhead map of the level around the camera is drawn: its names, metres across, texels across, metres above
/// the camera it is seen from and how far down it reaches, the step it moves by, and whether trees stand in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverheadShape {
  pub label: &'static str,
  pub cull_pass: &'static str,
  pub draw_pass: &'static str,
  /// Every pass a draw of it runs, as the timer names them, which the frame's scheduler measures its cost by.
  pub passes: &'static [&'static str],
  /// Milliseconds a texel of a draw costs before the timer measured one.
  pub seed: f32,
  pub width: f32,
  pub resolution: u32,
  pub height: f32,
  pub depth: f32,
  pub step: f32,
  /// Metres up or down the camera moves before it is seen from another height.
  pub height_step: f32,
  /// Frames a change of what the scene holds alone waits after the last draw before it draws again: a level streaming
  /// in redraws it no more often.
  pub content_delay: u32,
  pub is_tree_drawn: bool,
  /// Whether it is of the level's water alone, rather than what stands over it.
  pub is_water: bool,
}

impl OverheadShape {
  /// What stands over the rain, trees and all, as their leaves shelter: 64 m wide where only the streaks and the
  /// engine's wetting near the camera read it, three centimetres a texel, and fitted to the wet reach where the traced
  /// reflections want the wetness that far (`OverheadMap::fit_width`, 28 cm a texel at the default 250 m); its step a
  /// whole 64 texels of whatever width it has, so a thin edge never shifts in it as it moves.
  pub const RAIN_COVER: Self = Self {
    label: "rain cover",
    cull_pass: "rain cover cull",
    draw_pass: "rain cover",
    passes: &["rain cover cull", "rain cover", "rain cover pyramid"],
    seed: 7.0e-8,
    width: 64.0,
    resolution: 2048,
    height: 200.0,
    depth: 400.0,
    step: 2.0,
    height_step: 128.0,
    content_delay: 30,
    is_tree_drawn: true,
    is_water: false,
  };

  /// The level's own surface, which puddles are placed on: out past the farthest they are drawn, so its edge never
  /// shows, half a metre a texel, moving a whole 64 texels at a time so the static level never shifts in it; the trees
  /// left out, so the ground under their crowns is still the ground.
  pub const LEVEL_SURFACE: Self = Self {
    label: "level surface",
    cull_pass: "level surface cull",
    draw_pass: "level surface",
    passes: &["level surface cull", "level surface"],
    seed: 7.0e-8,
    width: 1024.0,
    resolution: 2048,
    height: 200.0,
    depth: 400.0,
    step: 32.0,
    height_step: 128.0,
    content_delay: 30,
    is_tree_drawn: false,
    is_water: false,
  };

  /// The level's water around the camera, laid as the level's surface is, which puddles stay above and away from.
  pub const LEVEL_WATER: Self = Self {
    label: "level water",
    cull_pass: "level water cull",
    draw_pass: "level water",
    passes: &["level water cull", "level water"],
    is_water: true,
    ..Self::LEVEL_SURFACE
  };
}
