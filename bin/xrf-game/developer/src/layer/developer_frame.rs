use xrf_renderer::{RenderLayer, RenderLayerTextures};

/// What one frame of the developer layer sends the renderer: its texture changes, to apply first and never to skip, and
/// the layer to draw.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeveloperFrame {
  pub textures: RenderLayerTextures,
  pub layer: RenderLayer,
}
