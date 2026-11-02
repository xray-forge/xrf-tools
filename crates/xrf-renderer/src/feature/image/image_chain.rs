use xrf_renderer_core::GraphTexture;

/// A view's frame once resolved: the image its viewport shows, and the frame as drawn that the passes after the resolve
/// read as the scene. What is laid over the resolved image (the camera, the enhanced streaks) is shown in its place,
/// and stands for the frame as drawn too where nothing rescaled it, so no history carries it.
#[derive(Clone, Copy)]
pub struct ImageChain {
  /// The frame at the drawn size: the resolved frame, or what was laid over it.
  pub resolved: GraphTexture,
  /// The image the viewport shows.
  pub shown: GraphTexture,
  /// The size of `shown`, the viewport's where it is rescaled.
  pub shown_size: (u32, u32),
  /// Whether `shown` is the frame rescaled to the viewport's size rather than the frame as drawn.
  pub is_rescaled: bool,
}

impl ImageChain {
  /// The chain with `layer` laid over what it shows, where one was drawn; as it was otherwise.
  pub fn lay(self, layer: Option<GraphTexture>) -> Self {
    let Some(layer) = layer else {
      return self;
    };

    Self {
      resolved: if self.is_rescaled { self.resolved } else { layer },
      shown: layer,
      ..self
    }
  }
}
