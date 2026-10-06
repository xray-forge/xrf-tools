/// How an overhead map of the level around the camera is drawn: its names, metres across, texels across, metres above
/// the camera it is seen from and how far down it reaches, the step it moves by, and whether trees stand in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverheadShape {
  pub label: &'static str,
  pub cull_pass: &'static str,
  pub draw_pass: &'static str,
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
  /// What stands over the rain: every streak's column falls within it, however far the camera is from its centre, and
  /// every surface the rain wets near the camera; six centimetres a texel, its step a whole 64 of them so a thin edge
  /// never shifts in it as it moves; trees and all, as their leaves shelter.
  pub const RAIN_COVER: Self = Self {
    label: "rain cover",
    cull_pass: "rain cover cull",
    draw_pass: "rain cover",
    width: 64.0,
    resolution: 1024,
    height: 60.0,
    depth: 120.0,
    step: 4.0,
    height_step: 4.0,
    content_delay: 0,
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
    is_water: true,
    ..Self::LEVEL_SURFACE
  };
}
