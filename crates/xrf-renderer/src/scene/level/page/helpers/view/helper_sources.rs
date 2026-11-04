use crate::contract::overlay::render_helper_options::RenderHelperOptions;
use crate::scene::level::page::helpers::level_helpers::LevelHelpers;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a view builds its helpers from: the scene's helpers and those whose objects sound, by their ids ascending, the
/// spawned objects standing in its scene, the view's options, and the layout the marks are bound by
/// (`StaticGBufferPass::get_marks_layout`).
#[derive(Clone, Copy)]
pub struct HelperSources<'a> {
  pub helpers: &'a LevelHelpers,
  pub sounding: &'a [u32],
  pub statics: &'a StaticScene,
  pub options: &'a RenderHelperOptions,
  pub marks_layout: &'a wgpu::BindGroupLayout,
}
