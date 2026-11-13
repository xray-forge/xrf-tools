use std::collections::HashMap;

use xrf_editable_level::EditableLightObject;
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::Hemisphere;
use xrf_level::{BuildPrjDynamicLight, BuildPrjLightControl, BuildPrjName, LevelDynamicLight};
use xrf_math::{Affine3d, Color, Matrix4x4, Vector2d, Vector3d};

use crate::level::build::level_build_sectors::LevelBuildSectors;

/// The lights of a build: the layers they are controlled by (`SceneBuilder::BuildLightControl`,
/// `BuilderRemote.cpp:918`), the static ones baked into the lightmaps and the dynamic ones the multi-use models take.
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

  /// The energy the sky is spread over (`xrHemisphereBuild(quality, 2.f, ...)`).
  const HEMI_ENERGY: f32 = 2.0;

  /// The sun's spread, three degrees (`deg2rad(3.f)`).
  const SUN_DISPERSION_DEGREES: f32 = 3.0;

  /// The sky of `quality`: one light per hemisphere direction under `$hemi`, else a single light straight down.
  pub(crate) fn add_hemisphere(&mut self, quality: u8) {
    let controller: u32 = self.get_control(Self::HEMI_CONTROL);

    if quality == 0 {
      self.static_lights.push(Self::directional(
        controller,
        Color::new(1.0, 1.0, 1.0, 1.0),
        Vector3d::new(0.0, -1.0, 0.0),
      ));
      return;
    }

    for light in Hemisphere::get_lights(quality, Self::HEMI_ENERGY) {
      self.static_lights.push(Self::directional(
        controller,
        Color::new(light.energy, light.energy, light.energy, 1.0),
        light.direction,
      ));
    }
  }

  /// The sun of `quality` along `direction` (pitch, then heading): a square of 1, 3, 4 or 7 static lights a side, meant
  /// to spread over three degrees, under `$sun`, sharing its energy, and one dynamic light along it.
  ///
  /// # Errors
  ///
  /// Returns an error for another quality, which the Build throws on.
  pub(crate) fn add_sun(&mut self, quality: u8, direction: &Vector2d) -> XrfResult {
    let samples: u16 = match quality {
      0 => 1,
      1 => 3,
      2 => 4,
      3 => 7,
      _ => {
        return Err(XrfError::new_invalid_error(format!(
          "Sun quality {quality} is not one the build knows"
        )));
      }
    };
    let controller: u32 = self.get_control(Self::SUN_CONTROL);
    let energy: f32 = 1.0 / f32::from(samples * samples);
    let dispersion: f32 = Self::SUN_DISPERSION_DEGREES.to_radians();
    let step: f32 = dispersion / f32::from(samples);
    let (pitch_from, heading_from) = (direction.x - dispersion / 2.0, direction.y - dispersion / 2.0);
    // The source's `float x = mn_x+x*da;` reads the variable it declares, so the shipped editor takes each sample from
    // the one before, from zero, the heading carried across rows: the samples close on one direction rather than
    // spreading over the square. `l01_test`'s `build.prj` holds exactly these.
    let (mut pitch, mut heading): (f32, f32) = (0.0, 0.0);

    for _ in 0..samples {
      pitch = pitch_from + pitch * step;

      for _ in 0..samples {
        heading = heading_from + heading * step;
        self.static_lights.push(Self::directional(
          controller,
          Color::new(energy, energy, energy, 1.0),
          Self::get_direction(heading, pitch),
        ));
      }
    }

    self.dynamic_lights.push(BuildPrjDynamicLight {
      light: Self::directional(
        controller,
        Color::new(1.0, 1.0, 1.0, 1.0),
        Self::get_direction(direction.y, direction.x),
      ),
      sector_slots: [0; BuildPrjDynamicLight::MAXIMUM_SECTORS],
      sector_count: 0,
    });

    Ok(())
  }

  /// A scene light under the control `controls` name for its index: baked where it lights static geometry, spread
  /// over its fuzzy points where it has them, and dynamic, reaching the sectors its range does, where it lights
  /// dynamic models.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, for a light lighting nothing, of a control the tool does not name, of a
  /// kind other than point, or dynamic and reaching no sector.
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
      direction: Self::get_direction(rotation.y, rotation.x),
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

  /// The layers as `build.prj` stores them, none holding the frames the format leaves room for.
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

  /// The index of the layer `name`, added last where none is named so.
  fn get_control(&mut self, name: &str) -> u32 {
    let index: usize = self.controls.iter().position(|held| held == name).unwrap_or_else(|| {
      self.controls.push(name.to_owned());
      self.controls.len() - 1
    });

    u32::try_from(index).unwrap_or(u32::MAX)
  }

  /// A directional light of `diffuse` along `direction`, the rest of it zero.
  fn directional(controller: u32, diffuse: Color, direction: Vector3d) -> LevelDynamicLight {
    LevelDynamicLight {
      controller,
      kind: LevelDynamicLight::KIND_DIRECTIONAL,
      diffuse,
      specular: Color::new(0.0, 0.0, 0.0, 0.0),
      ambient: Color::new(0.0, 0.0, 0.0, 0.0),
      position: Vector3d::ZERO,
      direction,
      range: 0.0,
      falloff: 0.0,
      attenuation_0: 0.0,
      attenuation_1: 0.0,
      attenuation_2: 0.0,
      theta: 0.0,
      phi: 0.0,
    }
  }

  /// `Fvector::setHP(heading, pitch)`: the direction a rotation of that heading and pitch looks along.
  fn get_direction(heading: f32, pitch: f32) -> Vector3d {
    Affine3d::from_heading_pitch_bank(heading, pitch, 0.0).z_axis
  }
}
