use glam::{Mat4, Vec4};

use crate::contract::render_ambient_occlusion_method::RenderAmbientOcclusionMethod;
use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_indirect_light_mode::RenderIndirectLightMode;
use crate::contract::render_indirect_light_settings::RenderIndirectLightSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::bitmask_search::BitmaskSearch;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::vbao_uniform::VbaoUniform;

fn enhanced() -> RenderIndirectLightSettings {
  RenderIndirectLightSettings {
    mode: RenderIndirectLightMode::Enhanced,
    ..RenderIndirectLightSettings::default()
  }
}

fn options_with(method: RenderAmbientOcclusionMethod, indirect: RenderIndirectLightSettings) -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.features.ambient_occlusion.method = method;
  options.features.indirect_light = indirect;

  options
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
fn gathers_only_enhanced_with_an_intensity() {
  assert_eq!(
    RenderIndirectLightSettings::default().mode,
    RenderIndirectLightMode::Engine
  );
  assert_eq!(RenderIndirectLightSettings::default().intensity, 1.0);
  assert!(!RenderIndirectLightSettings::default().is_drawn());
  assert!(enhanced().is_drawn());
  assert!(
    !RenderIndirectLightSettings {
      intensity: 0.0,
      ..enhanced()
    }
    .is_drawn()
  );
}

// VBAO and the indirect light share one search; GTAO or no occlusion leaves the light a search of its own; nothing
// searches while neither is asked for, unlit or in wireframe.
#[test]
fn searches_once_for_whatever_the_bitmask_yields() {
  let engine: RenderIndirectLightSettings = RenderIndirectLightSettings::default();
  let search = |options: &RenderViewOptions| BitmaskSearch::new(options);
  let shared: RenderViewOptions = options_with(RenderAmbientOcclusionMethod::Vbao, enhanced());

  assert_eq!(search(&options_with(RenderAmbientOcclusionMethod::Gtao, engine)), None);
  assert_eq!(
    search(&options_with(RenderAmbientOcclusionMethod::Vbao, engine)),
    Some(BitmaskSearch {
      is_occluding: true,
      is_lit: false
    })
  );
  assert_eq!(
    search(&options_with(RenderAmbientOcclusionMethod::Gtao, enhanced())),
    Some(BitmaskSearch {
      is_occluding: false,
      is_lit: true
    })
  );
  assert_eq!(
    search(&shared),
    Some(BitmaskSearch {
      is_occluding: true,
      is_lit: true
    })
  );
  assert_eq!(search(&shared).map(|it| it.get_names().search), Some("vbao + indirect"));

  for quality in [
    RenderAmbientOcclusionQuality::Low,
    RenderAmbientOcclusionQuality::Medium,
    RenderAmbientOcclusionQuality::High,
    RenderAmbientOcclusionQuality::Ultra,
  ] {
    assert!(quality.get_indirect_steps() > 0);
  }

  let mut unoccluded: RenderViewOptions = shared.clone();

  unoccluded.features.ambient_occlusion = RenderAmbientOcclusionSettings {
    is_enabled: false,
    ..unoccluded.features.ambient_occlusion
  };

  assert_eq!(
    search(&unoccluded),
    Some(BitmaskSearch {
      is_occluding: false,
      is_lit: true
    })
  );
  assert_eq!(
    search(&unoccluded).map(|it| it.get_names().search),
    Some("indirect light")
  );

  let mut unlit: RenderViewOptions = shared.clone();

  unlit.mode.is_lit = false;

  assert_eq!(search(&unlit), None);

  let mut wireframe: RenderViewOptions = shared;

  wireframe.mode.is_wireframe = true;

  assert_eq!(search(&wireframe), None);
}

// Combine adds the light only where it is gathered.
#[test]
fn adds_the_light_only_where_it_is_gathered() {
  let mut options: RenderViewOptions = options_with(RenderAmbientOcclusionMethod::Gtao, enhanced());

  assert_eq!(lighting_with(&options).indirect.x, 1.0);

  options.mode.is_wireframe = true;

  assert_eq!(lighting_with(&options).indirect.x, 0.0);

  options.mode.is_wireframe = false;
  options.features.indirect_light = RenderIndirectLightSettings::default();

  assert_eq!(lighting_with(&options).indirect.x, 0.0);
}

// The light's history is carried only beside the occlusion's, and only while frames are gathered.
#[test]
fn carries_the_light_only_with_the_search_history() {
  let settings: RenderAmbientOcclusionSettings = RenderAmbientOcclusionSettings::default();
  let uniform =
    |histories: (bool, bool)| VbaoUniform::new(&settings, (1.5, 3.0), Mat4::IDENTITY, (800, 600), 0, histories);

  assert_eq!(uniform((true, true)).has_light_history, 1.0);
  assert_eq!(uniform((false, true)).has_light_history, 0.0);
  assert_eq!(uniform((true, false)).has_light_history, 0.0);
  assert_eq!(uniform((true, true)).intensity, 1.5);
  assert_eq!(uniform((true, true)).light_radius, 3.0);
  assert_eq!(
    VbaoUniform::new(&settings, (1.0, 0.5), Mat4::IDENTITY, (800, 600), 0, (true, true)).light_radius,
    settings.radius
  );
  assert_eq!(
    VbaoUniform::new(&settings, (-1.0, 0.5), Mat4::IDENTITY, (800, 600), 0, (true, true)).intensity,
    0.0
  );
}
