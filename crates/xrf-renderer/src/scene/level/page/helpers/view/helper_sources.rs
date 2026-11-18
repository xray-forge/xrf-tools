use crate::contract::overlay::render_helper_options::RenderHelperOptions;
use crate::scene::level::page::helpers::level_helpers::LevelHelpers;
use crate::scene::static_scene::static_scene::StaticScene;

/// Inputs a view builds its helpers from; `sounding` holds object ids ascending, and `marks_layout` comes from
/// `StaticGBufferPass::get_marks_layout`.
#[derive(Clone, Copy)]
pub(crate) struct HelperSources<'a> {
  pub helpers: &'a LevelHelpers,
  pub sounding: &'a [u32],
  pub statics: &'a StaticScene,
  pub options: &'a RenderHelperOptions,
  pub marks_layout: &'a wgpu::BindGroupLayout,
}
