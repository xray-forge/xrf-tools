use std::f32::consts::{FRAC_PI_2, PI};

use crate::sun::SunPosition;
use crate::weather::{WeatherDescriptor, WeatherTime};

/// `EPS_S`, what `fis_zero` compares against by default.
const EPS_S: f32 = 0.000_000_1;

/// Where a mix stands the sun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeatherSunSource<'a> {
  /// Lerped between the two keyframes' own directions: OpenXRay with its dynamic sun off.
  Authored,
  /// OpenXRay's astronomical sun, `calculate_dynamic_sun_dir`.
  Dynamic,
  /// Monolith's hourly table, `calculate_config_sun_dir`.
  Table(&'a [SunPosition]),
}

impl WeatherSunSource<'_> {
  /// `calculate_dynamic_sun_dir`: the direction at a time of day turned by an azimuth in radians, and how much of the
  /// sun's colour is left as it sets.
  pub fn dynamic(time: f32, azimuth: f32) -> ([f32; 3], f32) {
    let day: f32 = WeatherTime::DAY as f32;
    let g: f32 = ((360.0 / 365.25) * (180.0 + time / day)).to_radians();
    let declination: f32 = (0.396_372 - 22.913_27 * g.cos() + 4.025_43 * g.sin() - 0.387_205 * (2.0 * g).cos()
      + 0.051_967 * (2.0 * g).sin()
      - 0.154_527 * (3.0 * g).cos()
      + 0.084_798 * (3.0 * g).sin())
    .to_radians();
    let correction: f32 =
      0.004_297 + 0.107_029 * g.cos() - 1.837_877 * g.sin() - 0.837_378 * (2.0 * g).cos() - 2.340_475 * (2.0 * g).sin();
    let mut hour_angle: f32 = (time / (day / 24.0) - 12.0) * 15.0 - 30.4 + correction;

    if hour_angle > 180.0 {
      hour_angle -= 360.0;
    }

    if hour_angle < -180.0 {
      hour_angle += 360.0;
    }

    let latitude: f32 = 50.27f32.to_radians();
    let zenith: f32 = (latitude.sin() * declination.sin()
      + latitude.cos() * declination.cos() * hour_angle.to_radians().cos())
    .clamp(-1.0, 1.0)
    .acos();
    let across: f32 = zenith.sin() * latitude.cos();
    let azimuth_cos: f32 = if across.abs() < EPS_S {
      0.0
    } else {
      ((declination.sin() - latitude.sin() * zenith.cos()) / across).clamp(-1.0, 1.0)
    };
    let (lowest, full): (f32, f32) = (1f32.to_radians(), 3f32.to_radians());
    let elevation: f32 = (FRAC_PI_2 - zenith).max(lowest);
    let blend: f32 = ((elevation - lowest) / (full - lowest)).clamp(0.0, 1.0);
    let heading: f32 = azimuth_cos.acos() + azimuth;
    let heading: f32 = if hour_angle < 0.0 { 2.0 * PI - heading } else { heading };

    (WeatherDescriptor::direction_of(heading, -elevation), blend)
  }

  /// `calculate_config_sun_dir`: the table's two hours around a time of day, lerped by the minute.
  pub fn table(positions: &[SunPosition], time: f32) -> [f32; 3] {
    let at = |hour: usize| positions.get(hour % 24).copied().unwrap_or_default();
    let current: f32 = time / (WeatherTime::DAY as f32 / 24.0);
    let hour: f32 = current.floor();
    let weight: f32 = current - hour;
    let from: SunPosition = at(hour as usize);
    let to: SunPosition = at(hour as usize + 1);
    let altitude: f32 = from.altitude + (to.altitude - from.altitude) * weight;
    let longitude: f32 = from.longitude + (to.longitude - from.longitude) * weight;

    WeatherDescriptor::direction_of(altitude.to_radians(), longitude.to_radians())
  }
}
