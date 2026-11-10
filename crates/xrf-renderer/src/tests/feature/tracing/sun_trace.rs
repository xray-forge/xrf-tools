//! When a view traces the sun's shadow.

use glam::Vec4;

use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::resolve::resolve_decisions::ResolveDecisions;
use crate::feature::tracing::sun_trace::sun_trace_feature::SunTraceFeature;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// A lit view under ray reconstruction traces the sun while it has a colour, and not at a weather's night between the
/// sun and the moon, whose sun adds nothing for a shadow to take away.
#[test]
fn traces_the_sun_only_while_it_has_a_colour() {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.mode.is_wireframe = false;

  let resolve: ResolveDecisions = ResolveDecisions {
    noise_frame: 0,
    reference_sample: None,
    is_ray_reconstructed: true,
  };
  let lighting = |sun: Vec4| LightingUniform {
    sun,
    ..bytemuck::Zeroable::zeroed()
  };

  assert!(SunTraceFeature::is_asked(
    &options,
    &lighting(Vec4::new(1.0, 0.9, 0.8, 1.0)),
    resolve
  ));
  assert!(!SunTraceFeature::is_asked(
    &options,
    &lighting(Vec4::new(0.0, 0.0, 0.0, 1.0)),
    resolve
  ));
}
