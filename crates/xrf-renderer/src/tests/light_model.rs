use glam::{Mat4, Vec3, Vec4};

use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_light_model::RenderLightModel;
use crate::contract::render_tonemap_mode::RenderTonemapMode;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::{authored_colour, srgb_decoded, srgb_encoded};
use crate::pass::display_mode::DisplayMode;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;

fn physical() -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.features.light_model = RenderLightModel::Physical;

  options
}

fn lighting_with(options: &RenderViewOptions, lighting: &RenderLighting) -> LightingUniform {
  LightingUniform::new(
    lighting,
    Mat4::IDENTITY,
    options,
    &LightingFrame {
      is_adapting: false,
      sky_blend: 0.0,
      is_irradiance_up: false,
      clouds_time: 0.0,
      sun_sprite: Vec4::ZERO,
    },
  )
}

#[test]
fn the_engine_light_model_is_the_default() {
  assert_eq!(
    RenderViewOptions::default().features.light_model,
    RenderLightModel::Engine
  );
  assert!(!RenderViewOptions::default().is_light_physical());
  assert!(!RenderViewOptions::default().is_scene_linear());
}

// The physical light holds the scene as linear light whatever the tonemap asks, and neither is physical nor linear
// unlit or in wireframe, where nothing is lit.
#[test]
fn physical_light_holds_a_linear_scene_while_lit_and_solid() {
  let lit: RenderViewOptions = physical();

  assert_eq!(lit.features.tonemap, RenderTonemapMode::Engine);
  assert!(lit.is_light_physical());
  assert!(lit.is_scene_linear());

  let mut unlit: RenderViewOptions = physical();

  unlit.mode.is_lit = false;
  assert!(!unlit.is_light_physical());
  assert!(!unlit.is_scene_linear());

  let mut wireframe: RenderViewOptions = physical();

  wireframe.mode.is_wireframe = true;
  assert!(!wireframe.is_light_physical());
}

#[test]
fn srgb_decoding_inverts_the_encoding_and_continues_past_white() {
  for linear in [0.0, 0.002, 0.0031308, 0.05, 0.18, 0.5, 1.0] {
    assert!((srgb_decoded(srgb_encoded(linear)) - linear).abs() < 1e-5, "{linear}");
  }

  // An engine light brighter than white, as a lumscale of two makes the sun, decodes past one along the same curve.
  assert!(srgb_decoded(2.0) > 4.0);
}

#[test]
fn authored_colours_are_decoded_only_under_the_physical_light() {
  let colour: Vec3 = Vec3::new(1.0, 0.5, 0.25);

  assert_eq!(authored_colour(colour, false), colour);
  assert!((authored_colour(colour, true) - Vec3::new(1.0, srgb_decoded(0.5), srgb_decoded(0.25))).length() < 1e-6);
}

// The sun and the ambient reach the lit passes decoded, and the shaders are told; the fog, the sky's tint and the
// forward passes' values stay the weather's display values, each pass decoding them where it lights with them.
#[test]
fn the_lighting_decodes_what_it_lights_with_and_flags_the_shaders() {
  let lighting: RenderLighting = RenderLighting {
    sun_color: Vec3::new(1.0, 0.5, 0.25),
    hemisphere_color: Vec3::splat(0.5),
    ..RenderLighting::default()
  };
  let engine: LightingUniform = lighting_with(&RenderViewOptions::default(), &lighting);
  let physical: LightingUniform = lighting_with(&physical(), &lighting);

  assert_eq!(engine.engine.w, 0.0);
  assert_eq!(physical.engine.w, 1.0);
  assert!(physical.sun.y < engine.sun.y);
  assert!((physical.sun.y - srgb_decoded(engine.sun.y)).abs() < 1e-5);
  assert_eq!(physical.fog_color, engine.fog_color);
  assert_eq!(physical.sky, engine.sky);
  assert_eq!(physical.forward_sun, engine.forward_sun);
  assert_eq!(physical.forward_hemi, engine.forward_hemi);
}

// The engine's scene is shown as it is, or by the engine's curve where it holds linear light; the physical light's by
// its display transform, PBR Neutral unless the view asks for another.
#[test]
fn each_view_is_shown_by_its_light_models_display_mode() {
  let mut linear: RenderViewOptions = RenderViewOptions::default();

  linear.features.tonemap = RenderTonemapMode::Enhanced;
  assert_eq!(DisplayMode::of(&RenderViewOptions::default()), DisplayMode::AsIs);
  assert_eq!(DisplayMode::of(&linear), DisplayMode::Tonemapped);
  assert_eq!(DisplayMode::of(&physical()), DisplayMode::PbrNeutral);

  for (transform, mode) in [
    (RenderDisplayTransform::Engine, DisplayMode::EngineEncoded),
    (RenderDisplayTransform::Agx, DisplayMode::Agx),
    (RenderDisplayTransform::PbrNeutral, DisplayMode::PbrNeutral),
  ] {
    let mut options: RenderViewOptions = physical();

    options.features.display_transform = transform;
    assert_eq!(DisplayMode::of(&options), mode);
  }

  let mut unlit: RenderViewOptions = physical();

  unlit.mode.is_lit = false;
  assert_eq!(DisplayMode::of(&unlit), DisplayMode::AsIs);
}
