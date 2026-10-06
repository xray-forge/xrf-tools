//! The enhanced fog: its height fog in the lighting, and the scattering's stages.

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::contract::render_fog_mode::RenderFogMode;
use crate::contract::render_fog_settings::RenderFogSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_fog::RenderFog;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::fog_scattering_uniform::FogScatteringUniform;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;

fn enhanced() -> RenderFogSettings {
  RenderFogSettings {
    mode: RenderFogMode::Enhanced,
    ..RenderFogSettings::default()
  }
}

fn options_with(fog: RenderFogSettings) -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.show.is_fogged = true;
  options.features.fog = fog;

  options
}

fn fogged() -> RenderLighting {
  RenderLighting {
    fog: Some(RenderFog {
      color: Vec3::splat(0.5),
      distance: 300.0,
      density: 0.5,
      far_plane: 400.0,
    }),
    ..RenderLighting::default()
  }
}

fn height_fog(options: &RenderViewOptions, lighting: &RenderLighting) -> Vec4 {
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
  .height_fog
}

#[test]
fn ships_the_engines_fog_with_the_enhanced_strengths() {
  let defaults: RenderFogSettings = RenderFogSettings::default();

  assert_eq!(defaults.mode, RenderFogMode::Engine);
  assert_eq!(
    (
      defaults.height,
      defaults.density,
      defaults.sun_color,
      defaults.scattering
    ),
    (8.0, 1.3, 0.1, 0.7)
  );
  assert!(!defaults.is_enhanced() && !defaults.is_scattered());
  assert!(enhanced().is_scattered());
  assert!(
    !RenderFogSettings {
      scattering: 0.0,
      ..enhanced()
    }
    .is_scattered()
  );
}

// The height fog reaches the shaders only while enhanced, lit and fogged; the engine's fog is drawn as it was otherwise.
#[test]
fn writes_the_height_fog_only_where_it_is_drawn() {
  assert_eq!(
    height_fog(&options_with(enhanced()), &fogged()),
    Vec4::new(8.0, 1.3, 0.1, 1.0)
  );
  assert_eq!(
    height_fog(&options_with(RenderFogSettings::default()), &fogged()),
    Vec4::ZERO
  );
  assert_eq!(
    height_fog(&options_with(enhanced()), &RenderLighting::default()),
    Vec4::ZERO
  );

  let mut unfogged: RenderViewOptions = options_with(enhanced());

  unfogged.show.is_fogged = false;

  assert_eq!(height_fog(&unfogged, &fogged()), Vec4::ZERO);

  let mut unlit: RenderViewOptions = options_with(enhanced());

  unlit.mode.is_lit = false;

  assert_eq!(height_fog(&unlit, &fogged()), Vec4::ZERO);
}

// No pass runs while the engine's fog is asked for, nothing scatters, no fog is drawn, unlit or in wireframe.
#[test]
fn scatters_only_an_enhanced_fog_drawn_lit_and_solid() {
  let size: (u32, u32) = (1001, 503);
  let scattered: RenderViewOptions = options_with(enhanced());

  assert!(FogScatteringUniform::for_view(&scattered, true, size, 0.0).is_some());
  assert!(FogScatteringUniform::for_view(&scattered, false, size, 0.0).is_none());
  assert!(FogScatteringUniform::for_view(&options_with(RenderFogSettings::default()), true, size, 0.0).is_none());
  assert!(
    FogScatteringUniform::for_view(
      &options_with(RenderFogSettings {
        scattering: 0.0,
        ..enhanced()
      }),
      true,
      size,
      0.0
    )
    .is_none()
  );

  let mut wireframe: RenderViewOptions = scattered.clone();

  wireframe.mode.is_wireframe = true;

  assert!(FogScatteringUniform::for_view(&wireframe, true, size, 0.0).is_none());

  let mut unfogged: RenderViewOptions = scattered;

  unfogged.show.is_fogged = false;

  assert!(FogScatteringUniform::for_view(&unfogged, true, size, 0.0).is_none());
}

// A quarter, a half, then the frame's own size, rounding up; the strength held to one.
#[test]
fn blurs_to_a_quarter_and_a_half_before_scattering_at_full_size() {
  let stages: [FogScatteringUniform; 3] = FogScatteringUniform::stages((1001, 503), 1.5, 2.0);

  assert_eq!(
    stages.map(|it| it.size),
    [
      Vec2::new(251.0, 126.0),
      Vec2::new(501.0, 252.0),
      Vec2::new(1001.0, 503.0)
    ]
  );
  assert!(stages.iter().all(|it| it.intensity == 1.0 && it.time == 2.0));
}
