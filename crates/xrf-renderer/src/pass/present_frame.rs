use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_rect::RenderRect;

/// What one frame's present shows and where: the view asked for, the viewport's rectangle it is drawn into, and what
/// the scene it shows was drawn with.
#[derive(Clone, Copy, Debug, Default)]
pub struct PresentFrame {
  pub view: RenderDebugView,
  pub output: RenderRect,
  /// Whether the scene was upscaled into the output.
  pub is_upscaled: bool,
  /// Whether an ambient occlusion was searched, and the indirect light gathered.
  pub is_occluded: bool,
  pub is_indirect: bool,
  /// How many pixels a side the reflections are traced at, none where they are not.
  pub reflection_ratio: Option<u32>,
  /// The selection's outline colour, none for no outline.
  pub selection: Option<[f32; 3]>,
  /// Whether the engine's bloom is added.
  pub is_bloomed: bool,
}
