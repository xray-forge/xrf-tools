use glam::{Mat4, Vec3, Vec4};
use xrf_engine_target::XrayEngine;

use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::reflection_uniform::ReflectionUniform;

fn enhanced() -> RenderReflectionSettings {
  RenderReflectionSettings {
    mode: RenderReflectionMode::Enhanced,
    ..RenderReflectionSettings::default()
  }
}

fn options_with(reflections: RenderReflectionSettings) -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.features.reflections = reflections;

  options
}

fn lighting_of(engine: XrayEngine, rain: Option<f32>) -> RenderLighting {
  RenderLighting {
    engine,
    rain: rain.map(|density| RenderRainfall {
      color: Vec3::ONE,
      density,
    }),
    ..RenderLighting::default()
  }
}

fn lighting_uniform(options: &RenderViewOptions, lighting: &RenderLighting) -> LightingUniform {
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
fn traces_only_enhanced_with_an_intensity_and_a_distance() {
  let defaults: RenderReflectionSettings = RenderReflectionSettings::default();

  assert_eq!(defaults.mode, RenderReflectionMode::Engine);
  assert_eq!(defaults.intensity, 1.0);
  assert_eq!(defaults.distance, 60.0);
  assert_eq!(defaults.quality, RenderReflectionQuality::High);
  assert!(!defaults.is_drawn());
  assert!(enhanced().is_drawn());
  assert!(
    !RenderReflectionSettings {
      intensity: 0.0,
      ..enhanced()
    }
    .is_drawn()
  );
  assert!(
    !RenderReflectionSettings {
      distance: 0.0,
      ..enhanced()
    }
    .is_drawn()
  );
}

// Every pass is skipped while the engine's own reflections are asked for, unlit, in wireframe, or on Anomaly's
// shading while it is dry, which reflects nothing for a ray to replace.
#[test]
fn skips_every_pass_where_nothing_is_traced() {
  let vanilla_dry: RenderLighting = lighting_of(XrayEngine::Vanilla, None);
  let extended_dry: RenderLighting = lighting_of(XrayEngine::Extended, None);
  let extended_wet: RenderLighting = lighting_of(XrayEngine::Extended, Some(0.6));
  let traced: RenderViewOptions = options_with(enhanced());

  assert_eq!(
    ReflectionTrace::new(&options_with(RenderReflectionSettings::default()), &vanilla_dry),
    None
  );
  assert_eq!(
    ReflectionTrace::new(&traced, &vanilla_dry),
    Some(ReflectionTrace {
      quality: RenderReflectionQuality::High,
      intensity: 1.0,
    })
  );
  assert_eq!(ReflectionTrace::new(&traced, &extended_dry), None);
  assert_eq!(
    ReflectionTrace::new(&traced, &lighting_of(XrayEngine::Extended, Some(0.0))),
    None
  );
  assert!(ReflectionTrace::new(&traced, &extended_wet).is_some());

  let mut unlit: RenderViewOptions = traced.clone();

  unlit.mode.is_lit = false;

  assert_eq!(ReflectionTrace::new(&unlit, &vanilla_dry), None);

  let mut wireframe: RenderViewOptions = traced;

  wireframe.mode.is_wireframe = true;

  assert_eq!(ReflectionTrace::new(&wireframe, &vanilla_dry), None);
}

// Combine stands the reflections in for the cube only where they are traced, and reads them at the quality's size.
#[test]
fn stands_in_for_the_cube_only_where_traced() {
  let vanilla: RenderLighting = lighting_of(XrayEngine::Vanilla, None);
  let ultra: RenderViewOptions = options_with(RenderReflectionSettings {
    quality: RenderReflectionQuality::Ultra,
    ..enhanced()
  });

  assert_eq!(
    lighting_uniform(&options_with(enhanced()), &vanilla).reflections,
    Vec4::new(1.0, 2.0, 0.0, 0.0)
  );
  assert_eq!(
    lighting_uniform(&ultra, &vanilla).reflections,
    Vec4::new(1.0, 1.0, 0.0, 0.0)
  );
  assert_eq!(
    lighting_uniform(&options_with(enhanced()), &lighting_of(XrayEngine::Extended, None)).reflections,
    Vec4::ZERO
  );
  assert_eq!(
    lighting_uniform(&options_with(RenderReflectionSettings::default()), &vanilla).reflections,
    Vec4::ZERO
  );

  let present = |ratio: Option<u32>| {
    PresentUniform::new(
      RenderDebugView::Reflections,
      (false, false, ratio),
      false,
      0.0,
      RenderRect::default(),
      &RenderImageCorrections::default(),
      (None, false),
    )
  };

  assert_eq!(present(Some(2)).is_reflected, 1);
  assert_eq!(present(Some(2)).reflection_ratio, 2.0);
  assert_eq!(present(None).is_reflected, 0);
}

// Each step up the ladder takes more steps; only Ultra traces at the frame's own size.
#[test]
fn climbs_the_quality_ladder() {
  let steps: Vec<u32> = RenderReflectionQuality::ALL.iter().map(|it| it.get_steps()).collect();

  assert!(steps.windows(2).all(|pair| pair[0] < pair[1]));
  assert_eq!(
    RenderReflectionQuality::ALL.map(RenderReflectionQuality::get_ratio),
    [2, 2, 2, 1]
  );
}

#[test]
fn lays_the_traced_depth_over_the_viewport() {
  let half: ReflectionUniform = ReflectionUniform::new(&enhanced(), (2830, 1930), 3, true);

  assert_eq!(half.base.to_array(), [1415.0, 965.0]);
  assert_eq!(half.ratio, 2.0);
  assert_eq!(half.steps, RenderReflectionQuality::High.get_steps());
  assert_eq!(half.has_history, 1.0);
  assert_eq!(half.distance, 60.0);

  let held: ReflectionUniform = ReflectionUniform::new(
    &RenderReflectionSettings {
      intensity: 3.0,
      quality: RenderReflectionQuality::Ultra,
      ..enhanced()
    },
    (800, 600),
    0,
    false,
  );

  assert_eq!(held.intensity, 1.0);
  assert_eq!(held.base.to_array(), [800.0, 600.0]);
  assert_eq!(held.has_history, 0.0);
}
