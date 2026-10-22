use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::lighting::luminance::luminance;

use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_light_model::RenderLightModel;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_tonemap_mode::RenderTonemapMode;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::daylight::Daylight;
use crate::lighting::light_unit::PHYSICAL_LIGHT_UNIT;
use crate::lighting::light_units::LightUnits;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::{authored_colour, authored_light, srgb_decoded, srgb_encoded};
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::display_curves::{agx as agx_curve, engine_display, pbr_neutral};
use crate::pass::display_mode::DisplayMode;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::exposure_metering::ExposureMetering;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::picture_shares::PictureShares;
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
      metering: ExposureMetering::default(),
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
  assert_eq!(lit.get_physical_light_level(), Some(RenderLightLevel::Authored));

  let mut unlit: RenderViewOptions = physical();

  unlit.mode.is_lit = false;
  assert!(!unlit.is_light_physical());
  assert!(!unlit.is_scene_linear());
  assert_eq!(unlit.get_physical_light_level(), None);

  let mut wireframe: RenderViewOptions = physical();

  wireframe.mode.is_wireframe = true;
  assert!(!wireframe.is_light_physical());
}

// The camera tells the shaders the light's level: nought under the engine's light, one at the authored levels, two at
// the physical, whose lamps alone fall off by the inverse square (`are_levels_physical`).
#[test]
fn the_camera_tells_the_shaders_the_lights_level() {
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 1.0, 0.5, 100.0);
  let level = |level: Option<RenderLightLevel>| {
    CameraUniform::new(&view, RenderRect::default(), Vec4::ZERO)
      .with_physical_light(level)
      .output
      .w
  };

  assert_eq!(level(None), 0.0);
  assert_eq!(level(Some(RenderLightLevel::Authored)), 1.0);
  assert_eq!(level(Some(RenderLightLevel::Physical)), 2.0);
}

// The authored levels sum their diffuse light at the lights' gain and, unmetered, show the sky and the fog's colour alike;
// the engine's light and the physical levels sum nothing.
#[test]
fn only_the_authored_levels_sum_their_diffuse_light() {
  let lighting: RenderLighting = RenderLighting::default();
  let authored: LightingUniform = lighting_with(&physical(), &lighting);
  let mut levels: RenderViewOptions = physical();

  levels.features.light_level = RenderLightLevel::Physical;

  let gain: f32 = LightUnits::of(&physical(), &lighting, &SkyHarmonics::default(), None).light_gain;

  assert!((authored.summed_light.x - gain).abs() < 1e-6);
  assert_eq!(
    lighting_with(&RenderViewOptions::default(), &lighting).summed_light.x,
    0.0
  );
  assert_eq!(lighting_with(&levels, &lighting).summed_light.x, 0.0);
  assert_eq!(authored.pictures.x, authored.pictures.y);
}

// The lights' specular weight follows their light only at the physical levels; under the engine's light and at the
// authored levels it is the engine's over the authored colour, at the authored levels at the lights' gain and unit, so
// `summed_specular` takes the engine's weight back as it joins the diffuse in display values.
#[test]
fn only_the_physical_levels_weigh_the_specular_by_the_light() {
  let lighting: RenderLighting = RenderLighting {
    sun_color: Vec3::splat(0.1),
    ..RenderLighting::default()
  };
  let engine: RenderViewOptions = RenderViewOptions::default();
  let authored: RenderViewOptions = physical();
  let mut levels: RenderViewOptions = physical();

  levels.features.light_level = RenderLightLevel::Physical;

  assert!(!engine.is_specular_linear());
  assert!(!authored.is_specular_linear());
  assert!(levels.is_specular_linear());

  let weight: f32 = lighting_with(&engine, &lighting).sun.w;
  let units: LightUnits = LightUnits::of(&authored, &lighting, &SkyHarmonics::default(), None);
  let summed: f32 = lighting_with(&authored, &lighting).sun.w / (units.weather * units.light_gain);

  assert!(
    (summed / weight - 1.0).abs() < 1e-5,
    "{summed} against the engine's {weight}"
  );
  assert!(weight > lighting.get_sun_specular(&engine.features.light_scales, true));
}

// The engine's curve and the engine's own showing take no anchor; AgX and PBR Neutral each lift a darker frame's key
// further, the engine's curve near linear in the dark, where AgX's toe darkens, and bending a brighter key down more
// than PBR Neutral's linear run under its shoulder.
#[test]
fn only_agx_and_pbr_neutral_are_anchored() {
  let daylit: f32 = ExposureHistogramUniform::get_daylit_key(ExposureHistogramUniform::get_daylight_scale(0.0));

  for mode in [DisplayMode::AsIs, DisplayMode::Tonemapped, DisplayMode::EngineEncoded] {
    assert_eq!(mode.get_anchor_stops(daylit, 1.0), 0.0, "{mode:?}");
  }

  for mode in [DisplayMode::Agx, DisplayMode::PbrNeutral] {
    assert!(
      mode.get_anchor_stops(daylit / 64.0, 1.0) > mode.get_anchor_stops(daylit, 1.0),
      "{mode:?}"
    );
  }
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
// its display transform, the engine's curve unless the view asks for another.
#[test]
fn each_view_is_shown_by_its_light_models_display_mode() {
  let mut linear: RenderViewOptions = RenderViewOptions::default();

  linear.features.tonemap = RenderTonemapMode::Enhanced;
  assert_eq!(DisplayMode::of(&RenderViewOptions::default()), DisplayMode::AsIs);
  assert_eq!(DisplayMode::of(&linear), DisplayMode::Tonemapped);
  assert_eq!(DisplayMode::of(&physical()), DisplayMode::EngineEncoded);

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
// holds while it does not adapt, where a daylit exterior's average is shown, raised by the compensation's stops.
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
  let shown: f32 = ExposureHistogramUniform::MIDDLE_GREY * ExposureHistogramUniform::DAYLIT_STOPS.exp2();

  assert!((daylight * ExposureHistogramUniform::DAYLIT_LOG.exp2() - shown).abs() < 1e-5);
  assert!((ExposureHistogramUniform::get_daylight_scale(1.0) / daylight - 2.0).abs() < 1e-4);

  // The law exposes a daylit exterior at its daylight scale, and a scene a stop darker by less than a stop more.
  let land: f32 = ExposureHistogramUniform::DAYLIT_LOG;
  let law = |land: f32| ExposureHistogramUniform::get_law_scale(0.0, land);

  assert!((law(land) / daylight - 1.0).abs() < 1e-4);
  assert!((law(land - 1.0) / law(land) - (1.0 - ExposureHistogramUniform::KEPT_SHARE).exp2()).abs() < 1e-4);
}

// At the authored levels the sky's irradiance keeps its shape but takes the authored hemisphere's level and colour, each
// channel its own: an orange sunset sky lights the shade in the game's blue. A channel the sky all but lacks is held to
// a share of its luminance; the physical levels keep the sky's own.
#[test]
fn the_authored_levels_light_the_shade_in_the_authored_hemispheres_colour() {
  let authored: Vec3 = Vec3::new(0.3, 0.35, 0.45);
  let mut harmonics: SkyHarmonics = SkyHarmonics::default();

  harmonics.coefficients[0] = Vec4::new(1.0, 0.5, 0.2, 1.0);
  harmonics.coefficients[1] = Vec4::new(0.4, 0.2, 0.1, authored.x);
  harmonics.coefficients[2].w = authored.y;
  harmonics.coefficients[3].w = authored.z;

  let levelled: [Vec4; 9] = harmonics.at_level(RenderLightLevel::Authored, 1.0);

  assert!((levelled[0].truncate() - authored).length() < 1e-5, "{}", levelled[0]);
  assert!((levelled[1].truncate() - Vec3::new(0.4, 0.2, 0.1) * authored / Vec3::new(1.0, 0.5, 0.2)).length() < 1e-5);
  assert_eq!(
    harmonics.at_level(RenderLightLevel::Physical, 1.0)[0],
    harmonics.coefficients[0]
  );

  harmonics.coefficients[0] = Vec4::new(1.0, 0.5, 0.0, 1.0);

  assert!(harmonics.at_level(RenderLightLevel::Authored, 1.0)[1].z.is_finite());
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
      metering: ExposureMetering::default(),
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

  assert!(
    (luminance(authored[0].truncate()) - 0.6).abs() < 1e-5,
    "{}",
    authored[0]
  );
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

fn metered_frame(units: LightUnits, current: f32, metering: ExposureMetering) -> LightingFrame {
  LightingFrame {
    is_adapting: true,
    pre_exposure: PreExposure {
      current,
      previous: current,
    },
    metering,
    light_units: units,
    daylight: None,
    sky_blend: 0.0,
    is_irradiance_up: false,
    clouds_time: 0.0,
    sun_sprite: Vec4::ZERO,
    sky_harmonics: SkyHarmonics::default(),
  }
}

// The pictures take the whole exposure at the authored levels under the engine's curve, and wherever the sky is not
// metered. At the physical levels a night's land, lifted past the authored law, leaves its sky and fog shown as the law
// exposes that land in the authored levels' units, the sky's share of the pre-exposure the shares were set against, so
// the frame's exposure snapping ahead of it cannot carry the sky; under AgX and PBR Neutral, and whatever the grading's
// contrast, the sky shows at its metered level as the engine's curve shows it, the fog keeping the land's lift.
#[test]
fn the_pictures_show_as_the_authored_law_shows_them() {
  let lighting: RenderLighting = RenderLighting::default();
  let metering: ExposureMetering = ExposureMetering {
    key: 0.01,
    land: Some(-6.0),
    sky: Some(-9.0),
    ..ExposureMetering::default()
  };
  let authored: RenderViewOptions = physical();
  let units: LightUnits = LightUnits::of(&authored, &lighting, &SkyHarmonics::default(), None);

  assert_eq!(
    PictureShares::of(&authored, &lighting, &metered_frame(units, 0.01, metering)),
    PictureShares::WHOLE
  );

  let mut levels: RenderViewOptions = physical();

  levels.features.light_level = RenderLightLevel::Physical;

  let world: LightUnits = LightUnits {
    weather: 0.5,
    ..LightUnits::NONE
  };
  let lifted: PictureShares = PictureShares::of(&levels, &lighting, &metered_frame(world, 64.0, metering));
  let gain: f32 = LightUnits::get_key_gain(lighting.get_sun_color(&levels.features.light_scales));
  let law: f32 = ExposureHistogramUniform::get_law_scale(0.0, -6.0 + (PHYSICAL_LIGHT_UNIT * gain / 0.5).log2());

  assert!((0.5 * 64.0 * lifted.fog / (PHYSICAL_LIGHT_UNIT * law) - 1.0).abs() < 1e-4);
  assert!(lifted.fog < 1.0);
  assert_eq!(lifted.sky, lifted.fog);
  assert!(lifted.is_pre_exposed);
  assert_eq!(lifted.to_vec4().z, 1.0);
  assert!(
    PictureShares::of(
      &levels,
      &lighting,
      &metered_frame(world, 64.0, ExposureMetering::default())
    )
    .is_pre_exposed
  );

  let unmetered: ExposureMetering = ExposureMetering { sky: None, ..metering };
  let graded = |light: f32, contrast: f32| 0.18 * (light / 0.18).powf(contrast);
  let sky: f32 = (-9.0_f32).exp2() * units.weather * 0.01;

  for (transform, mode, curve) in [
    (
      RenderDisplayTransform::Agx,
      DisplayMode::Agx,
      agx_curve as fn(Vec3) -> Vec3,
    ),
    (RenderDisplayTransform::PbrNeutral, DisplayMode::PbrNeutral, pbr_neutral),
  ] {
    for contrast in [0.7, 1.0, 1.5] {
      let mut options: RenderViewOptions = physical();

      options.features.display_transform = transform;
      options.features.grading.contrast = contrast;

      let shares: PictureShares = PictureShares::of(&options, &lighting, &metered_frame(units, 0.01, metering));
      let land: f32 = mode.get_anchor_stops(0.01, contrast);
      let anchored: f32 = luminance(curve(Vec3::splat(graded(sky * shares.sky, contrast) * land.exp2())));
      let engine: f32 = luminance(engine_display(Vec3::splat(graded(sky, contrast))));

      assert_eq!(shares.fog, 1.0);
      assert!(
        (srgb_encoded(anchored) - srgb_encoded(engine)).abs() < 0.002,
        "{mode:?} by {contrast}: {anchored} against {engine}"
      );
      assert_eq!(
        PictureShares::of(&options, &lighting, &metered_frame(units, 0.01, unmetered)),
        PictureShares::WHOLE
      );
    }
  }
}
