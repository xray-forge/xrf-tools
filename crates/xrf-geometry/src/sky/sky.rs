use xrf_error::{XrfError, XrfResult};
use xrf_math::{Affine3d, Vector3d};

use crate::sky::hemisphere::Hemisphere;
use crate::sky::sky_light::SkyLight;

/// The sky and the sun the Level Editor generates for a build from the level options at a quality, 0 to 3
/// (`SceneBuilder::BuildHemiLights`, `BuildSun`), lit under the `$hemi` and `$sun` layers. A level compile at a draft
/// quality regenerates them the same way.
pub struct Sky;

impl Sky {
  /// The energy the sky is spread over (`xrHemisphereBuild(quality, 2.f, ...)`).
  pub(crate) const HEMI_ENERGY: f32 = 2.0;

  /// The sun's spread, three degrees (`deg2rad(3.f)`).
  pub(crate) const SUN_DISPERSION_DEGREES: f32 = 3.0;

  /// The sky of `quality`: one light per hemisphere direction, sharing `Self::HEMI_ENERGY`, else at quality 0 a
  /// single light of energy 1 straight down. Another quality has no table, and no light.
  pub fn get_hemisphere(quality: u8) -> Vec<SkyLight> {
    if quality == 0 {
      return vec![SkyLight {
        direction: Vector3d::new(0.0, -1.0, 0.0),
        energy: 1.0,
      }];
    }

    Hemisphere::get_lights(quality, Self::HEMI_ENERGY)
  }

  /// The sun of `quality` along `pitch` and `heading`: a square of 1, 3, 4 or 7 lights a side, meant to spread over
  /// `Self::SUN_DISPERSION_DEGREES`, sharing an energy of 1.
  ///
  /// # Errors
  ///
  /// Returns an error for a quality above 3, which the Build throws on.
  pub fn get_sun(quality: u8, pitch: f32, heading: f32) -> XrfResult<Vec<SkyLight>> {
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
    let energy: f32 = 1.0 / f32::from(samples * samples);
    let dispersion: f32 = Self::SUN_DISPERSION_DEGREES.to_radians();
    let step: f32 = dispersion / f32::from(samples);
    let (pitch_from, heading_from) = (pitch - dispersion / 2.0, heading - dispersion / 2.0);
    // The source's `float x = mn_x+x*da;` reads the variable it declares, so the shipped editor takes each sample from
    // the one before, from zero, the heading carried across rows: the samples close on one direction rather than
    // spreading over the square. `l01_test`'s `build.prj` holds exactly these.
    let (mut sample_pitch, mut sample_heading): (f32, f32) = (0.0, 0.0);
    let mut lights: Vec<SkyLight> = Vec::with_capacity(usize::from(samples * samples));

    for _ in 0..samples {
      sample_pitch = pitch_from + sample_pitch * step;

      for _ in 0..samples {
        sample_heading = heading_from + sample_heading * step;
        lights.push(SkyLight {
          direction: Self::get_direction(sample_heading, sample_pitch),
          energy,
        });
      }
    }

    Ok(lights)
  }

  /// `Fvector::setHP(heading, pitch)`: the direction a rotation of that heading and pitch looks along.
  pub fn get_direction(heading: f32, pitch: f32) -> Vector3d {
    Affine3d::from_heading_pitch_bank(heading, pitch, 0.0).z_axis
  }

  /// The pitch and heading of a unit `direction`, [`Self::get_direction`] undone, the heading in `[0, 2π)` as the
  /// level options hold it: [`Self::get_sun`] takes each sample from the heading's value, not its direction alone, so
  /// a heading a turn apart gives another sun.
  pub fn get_pitch_heading(direction: &Vector3d) -> (f32, f32) {
    (
      direction.y.clamp(-1.0, 1.0).asin(),
      (-direction.x).atan2(direction.z).rem_euclid(std::f32::consts::TAU),
    )
  }
}
