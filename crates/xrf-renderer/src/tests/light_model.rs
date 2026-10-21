use glam::{Mat4, Vec3, Vec4};

use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_light_model::RenderLightModel;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_tonemap_mode::RenderTonemapMode;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::daylight::Daylight;
use crate::lighting::light_unit::PHYSICAL_LIGHT_UNIT;
use crate::lighting::light_units::LightUnits;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::{authored_colour, authored_light, srgb_decoded, srgb_encoded};
use crate::pass::display_mode::DisplayMode;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::pre_exposure::PreExposure;
use crate::pass::sky_harmonics::SkyHarmonics;

fn physical() -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.features.light_model = RenderLightModel::Physical;

  options
}

fn lighting_with(options: &RenderViewOptions, lighting: &RenderLighting) -> LightingUniform {
  lighting_at(options, lighting, PreExposure::NONE)
}

fn lighting_at(options: &RenderViewOptions, lighting: &RenderLighting, pre_exposure: PreExposure) -> LightingUniform {
  LightingUniform::new(
    lighting,
    Mat4::IDENTITY,
    options,
    &LightingFrame {
      is_adapting: false,
      pre_exposure,
      light_units: LightUnits::of(options, lighting, &SkyHarmonics::default(), None),
      daylight: None,
      sky_blend: 0.0,
      is_irradiance_up: false,
      clouds_time: 0.0,
      sun_sprite: Vec4::ZERO,
      sky_harmonics: SkyHarmonics::default(),
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

// A light keeps its strength and has its hue decoded, so two lights stand to each other as the game balanced them: a
// grey keeps its value, a dim light keeps its share of a bright one, and a coloured one its luminance.
#[test]
fn authored_lights_keep_their_strength_and_decode_their_hue() {
  let luminance = |colour: Vec3| colour.dot(Vec3::new(0.2126, 0.7152, 0.0722));
  let warm: Vec3 = Vec3::new(2.2, 1.76, 1.43);
  let lit: Vec3 = authored_light(warm, true);

  assert_eq!(authored_light(warm, false), warm);
  assert!((authored_light(Vec3::splat(0.04), true) - Vec3::splat(0.04)).length() < 1e-6);
  assert!((luminance(lit) - luminance(warm)).abs() < 1e-5);
  assert!((lit.y / lit.x - srgb_decoded(0.8)).abs() < 1e-5);
  assert_eq!(authored_light(Vec3::ZERO, true), Vec3::ZERO);
}

// The sun and the ambient reach the lit passes as light, their hue decoded and their strength kept, in the physical
// light's units by the key light's gain, so the sun itself stands at its decoded luminance; the shaders are told; the
// fog, the sky's tint and the forward passes' values stay the weather's display values, each pass converting them where
// it lights with them.
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
  let gain: f32 = LightUnits::get_key_gain(engine.sun.truncate());
  let sun: Vec3 = physical.sun.truncate() / PHYSICAL_LIGHT_UNIT;
  let luminance = |colour: Vec3| colour.dot(Vec3::new(0.2126, 0.7152, 0.0722));

  assert!((sun - authored_light(engine.sun.truncate(), true) * gain).length() < 1e-5);
  assert!((luminance(sun) - luminance(authored_colour(engine.sun.truncate(), true))).abs() < 1e-5);
  assert_eq!(engine.pre_exposure.z, 1.0);
  assert_eq!(physical.pre_exposure.z, PHYSICAL_LIGHT_UNIT);
  assert_eq!(physical.fog_color, engine.fog_color);
  assert_eq!(physical.sky, engine.sky);
  assert_eq!(physical.forward_sun, engine.forward_sun);
  assert_eq!(physical.forward_hemi, engine.forward_hemi);
}

// The sun's colour and specular weight and the ambient arrive at the pre-exposure, which the shaders are given with the
// last frame's; what the shaders light with themselves (the fog, the sky's tint, the forward passes') does not carry it.
#[test]
fn the_lighting_arrives_at_the_pre_exposure() {
  let lighting: RenderLighting = RenderLighting {
    sun_color: Vec3::new(1.0, 0.5, 0.25),
    ambient_color: Vec3::splat(0.2),
    ..RenderLighting::default()
  };
  let pre_exposure: PreExposure = PreExposure {
    current: 4.0,
    previous: 2.0,
  };
  let plain: LightingUniform = lighting_with(&physical(), &lighting);
  let scaled: LightingUniform = lighting_at(&physical(), &lighting, pre_exposure);
  let unit: f32 = LightUnits::of(&physical(), &lighting, &SkyHarmonics::default(), None).weather;

  assert!((scaled.sun - plain.sun * 4.0).length() < 1e-5);
  assert!((scaled.ambient - plain.ambient * 4.0).length() < 1e-5);
  assert_eq!(scaled.pre_exposure, Vec4::new(4.0, 2.0, unit, unit));
  assert_eq!(plain.pre_exposure, Vec4::new(1.0, 1.0, unit, unit));
  assert_eq!(scaled.fog_color, plain.fog_color);
  assert_eq!(scaled.sky, plain.sky);
  assert_eq!(scaled.forward_sun, plain.forward_sun);
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

// A light of one in the engine's units is ten thousand cd/m² under the physical light at its authored levels, the
// weather's lights taking the key light's gain, and has no units under the engine's; the physical exposure starts, and
// holds while it does not adapt, where a daylit exterior's average is middle grey, raised by the compensation's stops.
#[test]
fn the_physical_light_is_in_cd_per_square_metre() {
  let lighting: RenderLighting = RenderLighting::default();
  let sun: Vec3 = lighting.get_sun_color(&RenderLightScales::default());

  assert_eq!(
    LightUnits::of(&RenderViewOptions::default(), &lighting, &SkyHarmonics::default(), None),
    LightUnits::NONE
  );
  assert_eq!(
    LightUnits::of(&physical(), &lighting, &SkyHarmonics::default(), None),
    LightUnits {
      weather: PHYSICAL_LIGHT_UNIT,
      local: PHYSICAL_LIGHT_UNIT,
      light_gain: LightUnits::get_key_gain(sun),
    }
  );

  let daylight: f32 = ExposureHistogramUniform::get_daylight_scale(0.0);
  let reference: f32 = ExposureHistogramUniform::REFERENCE_LOG.exp2();

  assert!((daylight * reference - ExposureHistogramUniform::MIDDLE_GREY).abs() < 1e-5);
  assert!((ExposureHistogramUniform::get_daylight_scale(1.0) / daylight - 2.0).abs() < 1e-4);
}

// The physical light lights the hemisphere by the sky's irradiance as the sky feature read it back, keeping whether the
// sky was projected: at the physical levels the sky's own times the look's hemisphere scale; at the authored levels the
// sky's shape at the authored hemisphere's level (the scale already in it); a sky too dark to have a shape evenly.
#[test]
fn the_sky_irradiance_stands_at_its_levels() {
  let mut options: RenderViewOptions = physical();

  options.features.light_scales.hemi = 2.0;

  let mut harmonics: SkyHarmonics = SkyHarmonics::default();

  harmonics.coefficients[0] = Vec4::new(0.5, 0.25, 0.125, 1.0);
  harmonics.coefficients[1] = Vec4::new(0.1, 0.2, 0.3, 0.6);
  harmonics.coefficients[2].w = 0.6;
  harmonics.coefficients[3].w = 0.6;

  let lit = |options: &RenderViewOptions, harmonics: SkyHarmonics| -> [Vec4; 9] {
    let frame: LightingFrame = LightingFrame {
      is_adapting: false,
      pre_exposure: PreExposure::NONE,
      light_units: LightUnits::NONE,
      daylight: None,
      sky_blend: 0.0,
      is_irradiance_up: false,
      clouds_time: 0.0,
      sun_sprite: Vec4::ZERO,
      sky_harmonics: harmonics,
    };

    LightingUniform::new(&RenderLighting::default(), Mat4::IDENTITY, options, &frame).sky_harmonics
  };

  options.features.light_level = RenderLightLevel::Physical;
  assert_eq!(lit(&options, harmonics)[0], Vec4::new(1.0, 0.5, 0.25, 1.0));

  options.features.light_level = RenderLightLevel::Authored;

  let authored: [Vec4; 9] = lit(&options, harmonics);
  let luminance = |colour: Vec4| colour.truncate().dot(Vec3::new(0.2126, 0.7152, 0.0722));

  assert!((luminance(authored[0]) - 0.6).abs() < 1e-5, "{}", authored[0]);
  assert!((authored[1].truncate() / authored[0].truncate() - Vec3::new(0.2, 0.8, 2.4)).length() < 1e-4);
  assert_eq!(authored[0].w, 1.0);

  harmonics.coefficients[0] = Vec4::new(0.0, 0.0, 0.0, 1.0);

  let dark: [Vec4; 9] = lit(&options, harmonics);

  assert_eq!(dark[0], Vec4::new(0.6, 0.6, 0.6, 1.0));
  assert_eq!(dark[1].truncate(), Vec3::ZERO);
}

// At the physical levels the sky's unit brings the projected sky to the world's sky on a level surface, the sun stands
// apart in the daylight, and the lamps come to a thousandth.
#[test]
fn the_physical_levels_bring_the_sky_to_the_worlds() {
  let mut options: RenderViewOptions = physical();

  options.features.light_level = RenderLightLevel::Physical;

  let lighting: RenderLighting = RenderLighting::default();
  let mut harmonics: SkyHarmonics = SkyHarmonics::default();

  harmonics.coefficients[0] = Vec4::new(0.5, 0.5, 0.5, 1.0);

  let daylight: Daylight = Daylight::of(&lighting, 0.0);
  let units: LightUnits = LightUnits::of(&options, &lighting, &harmonics, Some(&daylight));
  let zenith: f32 = harmonics.get_zenith_irradiance(options.features.light_scales.hemi);

  assert!((std::f32::consts::PI * zenith * units.weather - daylight.sky).abs() / daylight.sky < 1e-4);
  assert_eq!(units.local, PHYSICAL_LIGHT_UNIT * LightUnits::PHYSICAL_LOCAL);
  // Before the sky was projected it lights at the unit.
  assert_eq!(
    LightUnits::of(&options, &lighting, &SkyHarmonics::default(), Some(&daylight)).weather,
    PHYSICAL_LIGHT_UNIT
  );
}
