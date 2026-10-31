use xrf_renderer_core::RasterPassBuilder;

use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_impostor_parameters::StaticImpostorParameters;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What a view's static draws bind: each layout's parameters drawing its visible list, and the impostors'.
#[derive(Clone, Copy)]
pub struct StaticDraws {
  /// By `StaticLayout::get_index`.
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
  pub impostors: StaticImpostorParameters,
}

impl StaticDraws {
  /// Declares what every layout's draws read on a pass drawing them.
  pub fn declare<'g, 'a>(&self, builder: RasterPassBuilder<'g, 'a>) -> RasterPassBuilder<'g, 'a> {
    Self::declare_layouts(builder, &self.layouts)
  }

  /// Declares what every layout's draws and the impostors read on a pass drawing them all.
  pub fn declare_impostored<'g, 'a>(&self, builder: RasterPassBuilder<'g, 'a>) -> RasterPassBuilder<'g, 'a> {
    self.declare(builder).parameters(&self.impostors)
  }

  /// Declares what every layout's draws read on a pass drawing them.
  pub fn declare_layouts<'g, 'a>(
    builder: RasterPassBuilder<'g, 'a>,
    layouts: &[StaticDrawParameters; StaticLayout::COUNT],
  ) -> RasterPassBuilder<'g, 'a> {
    layouts
      .iter()
      .fold(builder, |builder, parameters| builder.parameters(parameters))
  }
}
