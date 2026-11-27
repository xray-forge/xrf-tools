use std::collections::HashMap;

use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::window::layer_pass::LayerPass;
use crate::window::render_window::RenderWindow;

/// What the windows are drawn into from the device's start, kept from the starting GPU into the running one: each
/// window's swapchain, and the layer pass whose layout the layers' textures are bound through.
pub(crate) struct WindowSurfaces {
  /// Each window's swapchain, by its host's key.
  pub swapchains: HashMap<u64, RenderWindow>,
  /// A native host's own interface over everything else of a window.
  pub layer: LayerPass,
}

impl WindowSurfaces {
  /// No window opened yet, its layers drawn with pipelines from `source`.
  pub(crate) fn new(source: PipelineSource<'_>) -> Self {
    Self {
      swapchains: HashMap::new(),
      layer: LayerPass::new(source),
    }
  }

  /// Builds the layer's pipelines again from a library reloaded since.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    self.layer.refresh(source);
  }
}
