use crate::contract::render_helper_options::RenderHelperOptions;
use crate::scene::level::helpers::level_helpers::LevelHelpers;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a view builds its helpers from: the scene's helpers, the spawned objects standing in its scene, the view's
/// options, and the layout the marks are bound by (`StaticGBufferPass::get_marks_layout`).
#[derive(Clone, Copy)]
pub struct HelperSources<'a> {
  pub helpers: &'a LevelHelpers,
  pub statics: &'a StaticScene,
  pub options: &'a RenderHelperOptions,
  pub marks_layout: &'a wgpu::BindGroupLayout,
}
