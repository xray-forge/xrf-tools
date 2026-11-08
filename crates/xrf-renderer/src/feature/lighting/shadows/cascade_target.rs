use xrf_renderer_core::GraphTexture;

use crate::pass::view::view_binding::ViewBinding;

/// A cascade of the sun's maps a frame draws casters into: the pass's name as the frame's timings know it, the maps and
/// the cascade's layer of them, and the cascade's view.
#[derive(Clone, Copy)]
pub(crate) struct CascadeTarget<'a> {
  pub name: &'static str,
  pub maps: GraphTexture,
  pub layer: u32,
  pub view: &'a ViewBinding,
}
