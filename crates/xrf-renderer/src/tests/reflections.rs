use glam::{Mat4, Vec4};
use xrf_engine_target::XrayEngine;

use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_reflection_trace_quality::RenderReflectionTraceQuality;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::scene::level::feature::reflection_history::ReflectionHistory;

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

fn trace_of(reflections: RenderReflectionSettings) -> ReflectionTrace {
  ReflectionTrace::new(&options_with(reflections)).expect("Expect the settings to trace")
}

fn lighting_of(engine: XrayEngine) -> RenderLighting {
  RenderLighting {
    engine,
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
  assert_eq!(defaults.distance, 150.0);
  assert_eq!(defaults.quality, RenderReflectionQuality::High);
  assert_eq!(defaults.reach, 300.0);
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

// Every pass is skipped while the engine's own reflections are asked for, unlit or in wireframe; either engine traces
// wet or dry.
#[test]
fn skips_every_pass_where_nothing_is_traced() {
  let traced: RenderViewOptions = options_with(enhanced());

  assert_eq!(
    ReflectionTrace::new(&options_with(RenderReflectionSettings::default())),
    None
  );
  assert_eq!(
    ReflectionTrace::new(&traced),
    Some(ReflectionTrace {
      quality: RenderReflectionQuality::High,
      intensity: 1.0,
      sky_gain: 1.25,
      is_raw: false,
    })
  );

  let mut unlit: RenderViewOptions = traced.clone();

  unlit.mode.is_lit = false;

  assert_eq!(ReflectionTrace::new(&unlit), None);

  let mut wireframe: RenderViewOptions = traced;

  wireframe.mode.is_wireframe = true;

  assert_eq!(ReflectionTrace::new(&wireframe), None);
}

// Combine blends the reflections in only where they are traced, reads them at the quality's size and scales the share
// by the intensity, on either engine, wet or dry, with the sky's highlights expanded as light by the settings' gain.
#[test]
fn blends_reflections_in_only_where_traced() {
  let vanilla: RenderLighting = lighting_of(XrayEngine::Vanilla);
  let ultra: RenderViewOptions = options_with(RenderReflectionSettings {
    quality: RenderReflectionQuality::Ultra,
    probe: RenderReflectionProbe::Fine,
    ..enhanced()
  });

  assert_eq!(
    lighting_uniform(&options_with(enhanced()), &vanilla).reflections,
    Vec4::new(1.0, 2.0, 1.0, 1.25)
  );
  assert_eq!(
    lighting_uniform(&ultra, &vanilla).reflections,
    Vec4::new(1.0, 1.0, 1.0, 1.25)
  );
  assert_eq!(
    lighting_uniform(&options_with(enhanced()), &lighting_of(XrayEngine::Extended)).reflections,
    Vec4::new(1.0, 2.0, 1.0, 1.25)
  );
  assert_eq!(
    lighting_uniform(
      &options_with(RenderReflectionSettings {
        intensity: 2.5,
        ..enhanced()
      }),
      &vanilla
    )
    .reflections,
    Vec4::new(1.0, 2.0, 2.5, 1.25)
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

// Each step up the ladder lets a ray cross more of the depth pyramid's cells, and only Ultra traces at the frame's own
// size.
#[test]
fn climbs_the_quality_ladder() {
  let crossings: Vec<u32> = RenderReflectionQuality::ALL
    .iter()
    .map(|it| it.get_crossings())
    .collect();

  assert!(crossings.windows(2).all(|pair| pair[0] < pair[1]));
  assert_eq!(
    RenderReflectionQuality::ALL.map(RenderReflectionQuality::get_ratio),
    [2, 2, 2, 1]
  );
}

#[test]
fn writes_the_settings_the_trace_reads() {
  let high: ReflectionUniform = ReflectionUniform::new(&enhanced(), trace_of(enhanced()), 7, (true, true), true);

  assert_eq!(high.crossings, RenderReflectionQuality::High.get_crossings());
  assert_eq!(high.frame, 7);
  assert_eq!(high.ratio, 2.0);
  assert_eq!(high.has_history, 1.0);
  assert_eq!(high.has_source, 1.0);
  assert_eq!(high.distance, 150.0);
  assert_eq!(high.is_traced, 0.0, "tracing is opted into");
  assert_eq!(high.trace_turns, 2);

  let ultra_settings: RenderReflectionSettings = RenderReflectionSettings {
    intensity: 3.0,
    quality: RenderReflectionQuality::Ultra,
    probe: RenderReflectionProbe::Fine,
    is_traced: true,
    trace_quality: RenderReflectionTraceQuality::Ultra,
    ..enhanced()
  };
  let ultra: ReflectionUniform =
    ReflectionUniform::new(&ultra_settings, trace_of(ultra_settings), 0, (false, false), true);

  assert_eq!(ultra.intensity, 3.0);
  assert_eq!(ultra.ratio, 1.0);
  assert_eq!(ultra.has_history, 0.0);
  assert_eq!(ultra.has_source, 0.0);
  assert_eq!((ultra.is_traced, ultra.trace_turns), (1.0, 1));

  let asked: RenderReflectionSettings = RenderReflectionSettings {
    is_traced: true,
    ..enhanced()
  };

  assert_eq!(
    ReflectionUniform::new(&asked, trace_of(asked), 0, (false, false), false).is_traced,
    0.0,
    "a device without ray queries traces nothing, asked or not"
  );
}

// Taken raw under DLSS ray reconstruction, every pixel traces every frame at the frame's own size, whatever the quality
// asks; the settings' crossings and distance still hold.
#[test]
fn traces_every_pixel_every_frame_taken_raw() {
  let settings: RenderReflectionSettings = RenderReflectionSettings {
    is_traced: true,
    trace_quality: RenderReflectionTraceQuality::Low,
    ..enhanced()
  };
  let raw: ReflectionTrace = trace_of(settings).with_raw(true);
  let uniform: ReflectionUniform = ReflectionUniform::new(&settings, raw, 3, (false, true), true);

  assert_eq!(trace_of(settings).get_ratio(), 2);
  assert_eq!(raw.get_ratio(), 1);
  assert_eq!((uniform.ratio, uniform.trace_turns), (1.0, 1));
  assert_eq!(uniform.crossings, RenderReflectionQuality::High.get_crossings());
  assert_eq!(uniform.distance, 150.0);
  assert_eq!(trace_of(settings).with_raw(false), trace_of(settings));
}

// A raw trace's list holds half its pixels at the least and grows at once past what its frames asked, in whole steps,
// to a ray a pixel at most; it shrinks only once what they ask falls to half its room.
#[test]
fn sizes_a_raw_lists_room_to_what_its_frames_ask() {
  let traced: u32 = 2869 * 1484;
  let least: u32 = traced.div_ceil(ReflectionHistory::RAW_REQUEST_SHARE);

  assert_eq!(ReflectionHistory::get_raw_capacity(traced, least, 0), least);

  let grown: u32 = ReflectionHistory::get_raw_capacity(traced, least, traced * 2 / 3);

  assert!((grown > traced * 2 / 3 && grown.is_multiple_of(65_536)) || grown == traced);
  assert!(grown <= traced);
  assert_eq!(ReflectionHistory::get_raw_capacity(traced, grown, traced / 2), grown);
  assert_eq!(ReflectionHistory::get_raw_capacity(traced, grown, traced * 2), traced);
  assert_eq!(ReflectionHistory::get_raw_capacity(traced, traced, 1000), least);
}
