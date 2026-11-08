use crate::contract::view::output::render_scale::RenderScale;
use crate::contract::view::render_view_options::RenderViewOptions;

/// How fast the faces' exposure adapts while they are metered in turn: slow enough to settle on all six rather than
/// the last one drawn, at the step a capture's frames advance it by.
const METERING_ADAPTATION: f32 = 3.0;

/// What a cubemap's face is drawn with: the viewport's whole look at the face's own size.
pub(crate) struct CubeCaptureLook;

impl CubeCaptureLook {
  /// The options a face is drawn with, from the viewport's `options`: a lens flare would end at the face's edge, and
  /// the camera's vignette, blur, grain and aberration are each face's own, which would part the faces at every seam.
  pub(crate) fn apply(options: &RenderViewOptions) -> RenderViewOptions {
    let mut look: RenderViewOptions = options.clone();

    look.show.is_lens_flared = false;
    look.features.camera = look.features.camera.without_effects();
    look.show.is_calibration_visible = false;
    look.output.upscaling.scale = RenderScale::Native;
    look.output.upscaling.is_ray_reconstructed = false;
    look.output.output_height = None;
    look.output.output_scale = 1.0;

    look
  }

  /// The options the faces are metered with: a face's, its exposure adapting slowly over all six.
  pub(crate) fn apply_metering(options: &RenderViewOptions) -> RenderViewOptions {
    let mut look: RenderViewOptions = Self::apply(options);

    look.features.exposure.adaptation = METERING_ADAPTATION;

    look
  }
}
