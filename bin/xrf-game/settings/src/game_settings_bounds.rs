use xrf_renderer::{RenderViewOptions, RenderViewOutput};

use crate::game_settings::GameSettings;

impl GameSettings {
  /// The view's options these settings draw with: its features and upscaling, the rest the renderer's defaults.
  pub fn get_view_options(&self) -> RenderViewOptions {
    RenderViewOptions {
      features: self.view.clone(),
      output: RenderViewOutput {
        upscaling: self.upscaling,
        ..RenderViewOutput::default()
      },
      ..RenderViewOptions::default()
    }
  }

  /// Holds every setting the renderer bounds to the bounds it holds them to as it takes them, so the settings read as
  /// they are drawn with. The window's settings are not the renderer's and stay as they are.
  pub fn hold_to_bounds(&mut self) {
    let mut options: RenderViewOptions = self.get_view_options();

    options.hold_to_bounds();
    self.render.hold_to_bounds();
    self.view = options.features;
    self.upscaling = options.output.upscaling;
  }
}
