use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Sky, SkyLight};
use xrf_level::{BuildPrjFile, LevelDynamicLight, LevelLight, LevelLightsChunk, LevelLightsFile};
use xrf_math::{Color, Vector3d};

use crate::light::bake::light_quality::LightQuality;

/// A level's light layers as the compilers light by them, `build.lights`, made from `build.prj`.
pub(crate) struct LightLayers;

impl LightLayers {
  /// `build.lights` (`Build.cpp`): the `$static`, `$hemi` and `$sun` layers' lights as xrLC loads them
  /// (`Build_Load.cpp`), chunks 0, 1 and 2, the sky and the sun regenerated where `quality` asks for another
  /// quality than `build.prj`'s ([`Self::get_sky_layer`]).
  ///
  /// # Errors
  ///
  /// Returns an error for a layer `build.prj` lacks or that holds no light, and for a sun the sky cannot place.
  pub(crate) fn build(project: &BuildPrjFile, quality: LightQuality) -> XrfResult<LevelLightsFile> {
    let layer = |name: &str| -> XrfResult<(u32, Vec<LevelDynamicLight>)> {
      let control: u32 = project
        .light_controls
        .iter()
        .position(|control| control.name.value.eq_ignore_ascii_case(name))
        .and_then(|control| u32::try_from(control).ok())
        .ok_or_else(|| XrfError::new_invalid_error(format!("build.prj has no {name} light layer")))?;
      let lights: Vec<LevelDynamicLight> = project
        .static_lights
        .iter()
        .filter(|light| light.controller == control)
        .cloned()
        .collect();

      if lights.is_empty() {
        return Err(XrfError::new_invalid_error(format!(
          "build.prj's {name} light layer holds no light"
        )));
      }

      Ok((control, lights))
    };
    let (_, static_lights) = layer("$static")?;
    let (hemi_control, mut hemi) = layer("$hemi")?;
    let (sun_control, mut sun) = layer("$sun")?;

    if let Some(sky) = quality.get_sky_quality() {
      let (pitch, heading) = Sky::get_pitch_heading(&Self::get_sun_direction(project, sun_control, &sun));

      hemi = Self::get_sky_layer(hemi, hemi_control, Sky::get_hemisphere(sky));
      sun = Self::get_sky_layer(sun, sun_control, Sky::get_sun(sky, pitch, heading)?);
    }

    let chunk = |id: u32, lights: &[LevelDynamicLight]| LevelLightsChunk::Lights {
      id,
      lights: lights.iter().map(Self::get_light).collect(),
    };

    Ok(LevelLightsFile {
      chunks: vec![chunk(0, &static_lights), chunk(1, &hemi), chunk(2, &sun)],
    })
  }

  /// A sky or sun layer at the quality `generated` holds. The editor generates the layer's directional lights and
  /// takes the scene's lights under the layer as point lights alone (`LevelBuildLights`), so the directional ones are
  /// replaced by `generated` under `controller` and the scene's kept, unless they are as many as `generated` already.
  fn get_sky_layer(held: Vec<LevelDynamicLight>, controller: u32, generated: Vec<SkyLight>) -> Vec<LevelDynamicLight> {
    if held.iter().filter(|light| light.is_sun()).count() == generated.len() {
      return held;
    }

    generated
      .iter()
      .map(|light| {
        LevelDynamicLight::new_directional(
          controller,
          Color::new(light.energy, light.energy, light.energy, 1.0),
          light.direction,
        )
      })
      .chain(held.into_iter().filter(|light| !light.is_sun()))
      .collect()
  }

  /// The way the sun's light travels: the directional dynamic light under `$sun` the editor writes along it, else the
  /// mean of the layer's directional `lights`, which spread three degrees about it.
  fn get_sun_direction(project: &BuildPrjFile, control: u32, lights: &[LevelDynamicLight]) -> Vector3d {
    if let Some(sun) = project
      .dynamic_lights
      .iter()
      .map(|light| &light.light)
      .find(|light| light.controller == control && light.is_sun())
    {
      return sun.direction;
    }

    lights
      .iter()
      .filter(|light| light.is_sun())
      .fold(Vector3d::ZERO, |sum, light| {
        Vector3d::new(
          sum.x + light.direction.x,
          sum.y + light.direction.y,
          sum.z + light.direction.z,
        )
      })
      .normalize_or_zero()
  }

  /// One static light as xrLC loads it (`R_Light`): its energy the mean of its diffuse, its colour divided by it, its
  /// range a tenth longer, and its falloff from the attenuation at that range.
  fn get_light(light: &LevelDynamicLight) -> LevelLight {
    // xrLC's x87 arithmetic leaves the energy and colour a bit or two apart from these in their last place.
    let energy: f32 = (light.diffuse.r + light.diffuse.g + light.diffuse.b) / 3.0;
    let (diffuse, energy): (Vector3d, f32) = if energy.abs() > xrf_math::EPS_S {
      (
        Vector3d::new(
          light.diffuse.r / energy,
          light.diffuse.g / energy,
          light.diffuse.b / energy,
        ),
        energy,
      )
    } else {
      (Vector3d::ZERO, 0.0)
    };
    // xrLC flushes a subnormal result to zero, which a directional light's tiny range gives.
    let range: f32 = Some(light.range * 1.1)
      .filter(|range| !range.is_subnormal())
      .unwrap_or(0.0);
    let range_squared: f32 = range * range;
    let length_squared: f32 = light.direction.x * light.direction.x
      + light.direction.y * light.direction.y
      + light.direction.z * light.direction.z;
    let direction: Vector3d = if length_squared > f32::MIN_POSITIVE {
      let scale: f32 = (1.0 / length_squared).sqrt();

      Vector3d::new(
        light.direction.x * scale,
        light.direction.y * scale,
        light.direction.z * scale,
      )
    } else {
      light.direction
    };

    LevelLight {
      kind: if light.kind == LevelDynamicLight::KIND_DIRECTIONAL {
        LevelLight::KIND_DIRECTIONAL
      } else {
        LevelLight::KIND_POINT
      },
      level: 0,
      diffuse,
      position: light.position,
      direction,
      range,
      range_squared,
      falloff: 1.0
        / (range * (light.attenuation_0 + light.attenuation_1 * range + light.attenuation_2 * range_squared)),
      attenuation_constant: light.attenuation_0,
      attenuation_linear: light.attenuation_1,
      attenuation_quadratic: light.attenuation_2,
      energy,
      triangle: [Vector3d::ZERO; 3],
    }
  }
}
