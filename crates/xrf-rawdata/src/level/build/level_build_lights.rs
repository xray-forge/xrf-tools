use std::collections::HashMap;

use xrf_editable_level::EditableLightObject;
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Sky, SkyLight};
use xrf_level::{BuildPrjDynamicLight, BuildPrjLightControl, BuildPrjName, LevelDynamicLight};
use xrf_math::{Color, Matrix4x4, Vector2d, Vector3d};

use crate::level::build::level_build_sectors::LevelBuildSectors;

/// The lights of a build and the control layers they sit in (`SceneBuilder::BuildLightControl`): static ones baked
/// into the lightmaps, and dynamic ones for the multi-use models.
///
/// The sky and the sun are generated from the level options (`BuildHemiLights`, `BuildSun`); every other light is the
/// scene's (`BuildLight`).
#[derive(Default)]
pub(crate) struct LevelBuildLights {
  controls: Vec<String>,
  pub(crate) static_lights: Vec<LevelDynamicLight>,
  pub(crate) dynamic_lights: Vec<BuildPrjDynamicLight>,
}

impl LevelBuildLights {
  /// The layers xrLC looks lights up by (`LCONTROL_HEMI`, `LCONTROL_SUN`).
  pub(crate) const HEMI_CONTROL: &'static str = "$hemi";
  pub(crate) const SUN_CONTROL: &'static str = "$sun";

  /// The sky of `quality` under `$hemi`, [`Sky::get_hemisphere`].
  pub(crate) fn add_hemisphere(&mut self, quality: u8) {
    let controller: u32 = self.get_control(Self::HEMI_CONTROL);

    for light in Sky::get_hemisphere(quality) {
      self.static_lights.push(Self::get_sky_light(controller, &light));
    }
  }

  /// The sun of `quality` along `direction` (pitch, then heading) under `$sun`: its static lights ([`Sky::get_sun`])
  /// and one dynamic light.
  ///
  /// # Errors
  ///
  /// Returns an error for a quality above 3, which the Build throws on.
  pub(crate) fn add_sun(&mut self, quality: u8, direction: &Vector2d) -> XrfResult {
    let lights: Vec<SkyLight> = Sky::get_sun(quality, direction.x, direction.y)?;
    let controller: u32 = self.get_control(Self::SUN_CONTROL);

    for light in &lights {
      self.static_lights.push(Self::get_sky_light(controller, light));
    }

    self.dynamic_lights.push(BuildPrjDynamicLight {
      light: LevelDynamicLight::new_directional(
        controller,
        Color::new(1.0, 1.0, 1.0, 1.0),
        Sky::get_direction(direction.y, direction.x),
      ),
      sector_slots: [0; BuildPrjDynamicLight::MAXIMUM_SECTORS],
      sector_count: 0,
    });

    Ok(())
  }

  /// A scene light under its control layer: static where it lights static geometry, spread over its fuzzy points if
  /// it has them, and dynamic, in the sectors its range reaches, where it lights dynamic models.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, for a light lighting nothing, of a control `controls` lacks, of a kind
  /// other than point, or dynamic and reaching no sector.
  pub(crate) fn add_light(
    &mut self,
    light: &EditableLightObject,
    controls: &HashMap<u32, String>,
    sectors: &LevelBuildSectors,
  ) -> XrfResult {
    let name: &str = &light.object.name;

    if light.flags & (EditableLightObject::AFFECT_STATIC | EditableLightObject::AFFECT_DYNAMIC) == 0 {
      return Err(XrfError::new_invalid_error(format!(
        "Light '{name}' lights neither static nor dynamic geometry"
      )));
    }

    if light.kind != EditableLightObject::POINT {
      return Err(XrfError::new_invalid_error(format!(
        "Light '{name}' is of kind {}, and the build takes point lights alone",
        light.kind
      )));
    }

    let control: &str = controls.get(&light.control).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "Invalid light control name: light '{name}' names control {}",
        light.control
      ))
    })?;
    let controller: u32 = self.get_control(control);
    let [red, green, blue, alpha] = light.color;
    let rotation: &Vector3d = &light.object.transform.rotation;
    let base: LevelDynamicLight = LevelDynamicLight {
      controller,
      kind: LevelDynamicLight::KIND_POINT,
      diffuse: Color::new(
        red * light.brightness,
        green * light.brightness,
        blue * light.brightness,
        alpha,
      ),
      specular: Color::new(0.0, 0.0, 0.0, 0.0),
      ambient: Color::new(0.0, 0.0, 0.0, 0.0),
      position: light.object.transform.position,
      direction: Sky::get_direction(rotation.y, rotation.x),
      range: light.range,
      falloff: 0.0,
      attenuation_0: light.attenuation[0],
      attenuation_1: light.attenuation[1],
      attenuation_2: light.attenuation[2],
      theta: 0.0,
      phi: light.cone,
    };

    if light.flags & EditableLightObject::AFFECT_STATIC != 0 {
      match light.fuzzy.as_ref().filter(|fuzzy| !fuzzy.points.is_empty()) {
        Some(fuzzy) => {
          let diffuse: &Color = &base.diffuse;
          let magnitude: f32 = (diffuse.r * diffuse.r + diffuse.g * diffuse.g + diffuse.b * diffuse.b).sqrt();
          #[expect(clippy::cast_precision_loss, reason = "a light holds a few hundred points at most")]
          let share: f32 = magnitude / fuzzy.points.len() as f32;
          let matrix: Matrix4x4 = light.object.transform.to_matrix();

          for point in &fuzzy.points {
            self.static_lights.push(LevelDynamicLight {
              diffuse: Color::new(
                diffuse.r / magnitude * share,
                diffuse.g / magnitude * share,
                diffuse.b / magnitude * share,
                diffuse.a,
              ),
              position: matrix.transform_point(point),
              ..base.clone()
            });
          }
        }
        None => self.static_lights.push(base.clone()),
      }
    }

    if light.flags & EditableLightObject::AFFECT_DYNAMIC != 0 {
      let reached: Vec<u16> = sectors.get_light_sectors(&base.position, base.range);

      if reached.is_empty() {
        return Err(XrfError::new_invalid_error(format!(
          "Light '{name}' is dynamic and reaches no sector"
        )));
      }

      let mut sector_slots: [u16; BuildPrjDynamicLight::MAXIMUM_SECTORS] = [0; BuildPrjDynamicLight::MAXIMUM_SECTORS];

      sector_slots[..reached.len()].copy_from_slice(&reached);
      self.dynamic_lights.push(BuildPrjDynamicLight {
        light: base,
        sector_slots,
        sector_count: u32::try_from(reached.len()).unwrap_or(u32::MAX),
      });
    }

    Ok(())
  }

  /// The layers as `build.prj` stores them, with no frames.
  ///
  /// # Errors
  ///
  /// Returns an error when a name is not Windows-1251 or overfills its buffer.
  pub(crate) fn get_controls(&self) -> XrfResult<Vec<BuildPrjLightControl>> {
    self
      .controls
      .iter()
      .map(|name| {
        Ok(BuildPrjLightControl {
          name: BuildPrjName::new(name, BuildPrjName::SHORT_SIZE)?,
          data: Vec::new(),
        })
      })
      .collect()
  }

  /// The index of the layer `name`, appended when new.
  fn get_control(&mut self, name: &str) -> u32 {
    let index: usize = self.controls.iter().position(|held| held == name).unwrap_or_else(|| {
      self.controls.push(name.to_owned());
      self.controls.len() - 1
    });

    u32::try_from(index).unwrap_or(u32::MAX)
  }

  /// A generated sky or sun light, white at its energy.
  fn get_sky_light(controller: u32, light: &SkyLight) -> LevelDynamicLight {
    LevelDynamicLight::new_directional(
      controller,
      Color::new(light.energy, light.energy, light.energy, 1.0),
      light.direction,
    )
  }
}
