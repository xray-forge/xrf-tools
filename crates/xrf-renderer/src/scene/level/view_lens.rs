use std::borrow::Cow;

use glam::Vec3;

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_contact_shadow_mode::RenderContactShadowMode;
use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_show_flags::RenderShowFlags;
use crate::contract::render_view_features::RenderViewFeatures;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_volumetric_mode::RenderVolumetricMode;

/// What a view's lens lets it draw: the options a view is drawn with, once its camera is known, less what its lens
/// cannot draw right, whichever view it is.
///
/// A perspective draws everything. Parallel rays, a level map's or an editor's view from a side, have no eye: what
/// marches or looks up along the view from it (screen-space and traced reflections, froxels and shafts, contact
/// marches, the sky's directions, the lens flare), measures from it (the occlusion pyramid's rectangles, impostors'
/// facets) or models its lens (the upscalers) is turned off; their sun is one cascade over the box they see, and their
/// grass is planted out to its corners.
pub struct ViewLens;

impl ViewLens {
  /// `options` as `view`'s lens lets them be drawn: unchanged through a perspective.
  pub fn fit<'a>(options: &'a RenderViewOptions, view: &CameraView) -> Cow<'a, RenderViewOptions> {
    match view.get_orthographic_extent() {
      Some(extent) => Cow::Owned(Self::fit_parallel(options.clone(), extent)),
      None => Cow::Borrowed(options),
    }
  }

  /// `options` drawn through parallel rays seeing a box `extent` across, up and deep.
  fn fit_parallel(mut options: RenderViewOptions, extent: Vec3) -> RenderViewOptions {
    let show: &mut RenderShowFlags = &mut options.show;

    show.is_sky_visible = false;
    show.is_clouded = false;
    show.is_sky_hazed = false;
    show.is_lens_flared = false;
    show.is_sun_shafted = false;

    let features: &mut RenderViewFeatures = &mut options.features;

    features.reflections.mode = RenderReflectionMode::Engine;
    features.reflections.is_traced = false;
    features.reflections.is_water_reflected = false;
    features.volumetric.mode = RenderVolumetricMode::Engine;
    features.lights.is_volumetric = false;
    features.shadows.contact.mode = RenderContactShadowMode::Engine;
    features.lod.is_impostors = false;
    features.is_occlusion_culled = false;
    // One cascade holds the whole box whichever way the light comes: as wide as its diagonal.
    features.shadows.cascades = vec![extent.length()];
    features.shadows.is_staggered = false;
    // The ring is planted out to the box's corners, as far as the grass reaches at most.
    features.grass.radius = (extent.x.hypot(extent.y) * 0.5).ceil();

    // The upscalers need a perspective's lens; the temporal smoothing alone settles.
    if matches!(
      features.antialiasing,
      RenderAntialiasing::Fsr | RenderAntialiasing::Dlss
    ) {
      features.antialiasing = RenderAntialiasing::Taa;
    }

    options.output.upscaling.scale = RenderScale::Native;
    options.output.upscaling.is_ray_reconstructed = false;

    options
  }
}
