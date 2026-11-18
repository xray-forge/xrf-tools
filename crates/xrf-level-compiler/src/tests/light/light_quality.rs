//! The sky and the sun `build.lights` takes at each light quality.

use xrf_error::XrfResult;
use xrf_geometry::{Sky, SkyLight};
use xrf_level::{
  BuildPrjDynamicLight, BuildPrjFile, BuildPrjLightControl, BuildPrjName, BuildPrjParams, LevelDynamicLight,
  LevelLight, LevelLightRole, LevelLightsFile,
};
use xrf_math::{Color, Vector3d};

use crate::light::bake::light_quality::LightQuality;
use crate::light::source::light_layers::LightLayers;
use crate::light::vertex::light_models::LightModels;
use crate::tests::geometry::geometry_mesh::project;

/// The sun's pitch and heading in the editor's level options.
const SUN: (f32, f32) = (-0.8, 2.4);

/// A project lit as the editor writes it at `sky` quality: a lamp under `$static`, the sky under `$hemi` and a scene
/// lamp the level's author put there, the sun under `$sun` with its dynamic light along it.
pub(crate) fn lit(sky: u8) -> XrfResult<BuildPrjFile> {
  let light = |controller: u32, light: &SkyLight| {
    LevelDynamicLight::new_directional(
      controller,
      Color::new(light.energy, light.energy, light.energy, 1.0),
      light.direction,
    )
  };
  let lamp = |controller: u32| LevelDynamicLight {
    kind: LevelDynamicLight::KIND_POINT,
    range: 10.0,
    attenuation_0: 1.0,
    ..LevelDynamicLight::new_directional(controller, Color::new(1.0, 0.5, 0.25, 1.0), Vector3d::ZERO)
  };
  let (pitch, heading) = SUN;
  let mut project: BuildPrjFile = project(&[], &[])?;

  project.light_controls = ["$static", "$hemi", "$sun"]
    .into_iter()
    .map(|name| {
      Ok(BuildPrjLightControl {
        name: BuildPrjName::new(name, BuildPrjName::SHORT_SIZE)?,
        data: Vec::new(),
      })
    })
    .collect::<XrfResult<_>>()?;
  project.static_lights = std::iter::once(lamp(0))
    .chain(Sky::get_hemisphere(sky).iter().map(|sky| light(1, sky)))
    .chain(std::iter::once(lamp(1)))
    .chain(Sky::get_sun(sky, pitch, heading)?.iter().map(|sun| light(2, sun)))
    .collect();
  project.dynamic_lights = vec![BuildPrjDynamicLight {
    light: LevelDynamicLight::new_directional(2, Color::new(1.0, 1.0, 1.0, 1.0), Sky::get_direction(heading, pitch)),
    sector_slots: [0; BuildPrjDynamicLight::MAXIMUM_SECTORS],
    sector_count: 0,
  }];

  Ok(project)
}

/// The directions of a role's lights.
fn directions(lights: &LevelLightsFile, role: LevelLightRole) -> Vec<Vector3d> {
  lights
    .get_role_lights(role)
    .map(|light: &LevelLight| light.direction)
    .collect()
}

#[test]
fn keeps_build_prjs_sky_and_sun_at_scene_quality() -> XrfResult {
  let lights: LevelLightsFile = LightLayers::build(&lit(3)?, LightQuality::Scene)?;

  assert_eq!(lights.get_role_lights(LevelLightRole::Rgb).count(), 1);
  assert_eq!(lights.get_role_lights(LevelLightRole::Hemi).count(), 197);
  assert_eq!(lights.get_role_lights(LevelLightRole::Sun).count(), 49);

  Ok(())
}

#[test]
fn regenerates_the_sky_and_the_sun_at_quality_one_for_a_draft_along_the_same_sun() -> XrfResult {
  let draft: LevelLightsFile = LightLayers::build(&lit(3)?, LightQuality::Draft)?;
  // A project lit at quality 1 already keeps its own, which the editor generated as the draft regenerates them.
  let editor: LevelLightsFile = LightLayers::build(&lit(1)?, LightQuality::Draft)?;

  assert_eq!(draft.get_role_lights(LevelLightRole::Rgb).count(), 1);
  // The scene's lamp under `$hemi` stays, after the regenerated sky.
  assert_eq!(draft.get_role_lights(LevelLightRole::Hemi).count(), 27);
  assert_eq!(draft.get_role_lights(LevelLightRole::Sun).count(), 9);

  for role in [LevelLightRole::Hemi, LevelLightRole::Sun] {
    let (ours, theirs) = (directions(&draft, role), directions(&editor, role));

    assert_eq!(ours.len(), theirs.len());

    for (ours, theirs) in ours.iter().zip(&theirs) {
      assert!(ours.distance(theirs) < 1e-5, "{role:?}: {ours:?} against {theirs:?}");
    }
  }

  let energy: f32 = draft
    .get_role_lights(LevelLightRole::Sun)
    .map(|light| light.energy)
    .sum();

  assert!((energy - 1.0).abs() < 1e-5);

  Ok(())
}

#[test]
fn names_each_light_quality_on_a_command_line() {
  for quality in LightQuality::ALL {
    assert_eq!(LightQuality::from_name(quality.get_name()), Some(quality));
  }

  assert_eq!(LightQuality::default(), LightQuality::Scene);
}

#[test]
fn lights_the_models_from_one_sample_at_either_draft() -> XrfResult {
  // `ebqHigh`.
  let scene: BuildPrjParams = BuildPrjParams {
    quality: 1,
    ..project(&[], &[])?.params
  };
  let draft: BuildPrjParams = BuildPrjParams {
    quality: BuildPrjParams::QUALITY_DRAFT,
    ..scene
  };

  assert_eq!(
    LightQuality::Draft.get_model_samples(&scene),
    LightModels::DRAFT_SAMPLES
  );
  assert_eq!(
    LightQuality::Scene.get_model_samples(&draft),
    LightModels::DRAFT_SAMPLES
  );
  assert_eq!(LightQuality::Scene.get_model_samples(&scene), LightModels::SAMPLES);

  Ok(())
}
