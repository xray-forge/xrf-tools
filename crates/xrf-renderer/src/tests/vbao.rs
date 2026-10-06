use glam::{Mat4, Vec4};

use crate::contract::render_ambient_occlusion_method::RenderAmbientOcclusionMethod;
use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_ambient_occlusion_vbao_settings::{
  RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION, RenderAmbientOcclusionVbaoSettings,
};
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::vbao_uniform::VbaoUniform;

fn vbao(accumulation: u32) -> RenderAmbientOcclusionSettings {
  RenderAmbientOcclusionSettings {
    method: RenderAmbientOcclusionMethod::Vbao,
    vbao: RenderAmbientOcclusionVbaoSettings {
      accumulation,
      ..RenderAmbientOcclusionVbaoSettings::default()
    },
    ..RenderAmbientOcclusionSettings::default()
  }
}

fn lighting_with(options: &RenderViewOptions) -> LightingUniform {
  LightingUniform::new(
    &RenderLighting::default(),
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
fn searches_with_gtao_unless_asked_for_vbao() {
  assert!(!RenderAmbientOcclusionSettings::default().is_vbao());
  assert!(vbao(8).is_vbao());
  assert_eq!(vbao(0).vbao.get_accumulation(), 1);
  assert_eq!(
    vbao(1000).vbao.get_accumulation(),
    RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION
  );

  for quality in [
    RenderAmbientOcclusionQuality::Low,
    RenderAmbientOcclusionQuality::Medium,
    RenderAmbientOcclusionQuality::High,
    RenderAmbientOcclusionQuality::Ultra,
  ] {
    let (slices, steps) = quality.get_vbao_search();

    assert!(slices > 0 && steps > 0);
  }
}

// Accumulating, the noise turns with the frame and comes back to its start; gathering nothing, it holds still and
// nothing is carried.
#[test]
fn turns_its_noise_only_while_it_accumulates() {
  let uniform = |settings: &RenderAmbientOcclusionSettings, frame: u32| {
    VbaoUniform::new(settings, (0.0, 0.0), Mat4::IDENTITY, (800, 600), frame, (true, false))
  };
  let gathering: RenderAmbientOcclusionSettings = vbao(8);
  let still: RenderAmbientOcclusionSettings = vbao(1);

  assert_ne!(uniform(&gathering, 0).slice_noise, uniform(&gathering, 1).slice_noise);
  assert_ne!(uniform(&gathering, 0).step_noise, uniform(&gathering, 1).step_noise);
  assert_eq!(uniform(&gathering, 3).slice_noise, uniform(&gathering, 259).slice_noise);
  assert_eq!(uniform(&gathering, 0).has_history, 1.0);
  assert_eq!(uniform(&gathering, 0).frames, 8.0);
  assert_eq!(uniform(&still, 0).slice_noise, uniform(&still, 5).slice_noise);
  assert_eq!(uniform(&still, 5).has_history, 0.0);
  assert_eq!(uniform(&still, 0).reach, 120.0);
  assert_eq!(uniform(&still, 0).spread, 2.0 / 800.0);
}

// The occlusion darkens only where it is searched, and only VBAO bounces light back into it.
#[test]
fn bounces_light_only_where_the_vbao_is_drawn() {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.features.ambient_occlusion = vbao(8);

  let drawn: LightingUniform = lighting_with(&options);

  assert_eq!(drawn.params.w, 1.0);
  assert_eq!(drawn.params.x, 1.0);

  options.mode.is_wireframe = true;

  let wireframe: LightingUniform = lighting_with(&options);

  assert_eq!(wireframe.params.w, 0.0);
  assert_eq!(wireframe.params.x, 0.0);

  options.mode.is_wireframe = false;
  options.features.ambient_occlusion.method = RenderAmbientOcclusionMethod::Gtao;

  let gtao: LightingUniform = lighting_with(&options);

  assert_eq!(gtao.params.w, 1.0);
  assert_eq!(gtao.params.x, 0.0);
}
