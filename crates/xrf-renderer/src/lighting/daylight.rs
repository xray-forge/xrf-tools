use glam::Vec3;

use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::authored_colour;

/// The world's light for the physical light levels, from the weather's clock and its sun: the sun's or the moon's
/// illuminance at normal incidence through the atmosphere, and the clear or clouded sky's on a level surface, in lux.
/// The weather keeps what it alone knows: how much of the sun its clouds let through, which light is the moon, the
/// shape and colour of its sky.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Daylight {
  /// The sun's, or by night the moon's, illuminance at normal incidence, lux, in its linear colour.
  pub sun: Vec3,
  /// The sky's illuminance on a level surface, lux.
  pub sky: f32,
}

impl Daylight {
  /// The Zone's latitude, degrees north.
  const LATITUDE: f32 = 51.4;
  /// The sun's declination the shipped cycles' noon gives at that latitude: `w_clear1` stands its sun 53.4° up at
  /// 13:00, an early May day, which puts sunrise at 05:43, where the cycles' dawn hands over.
  const DECLINATION: f32 = 14.8;
  /// The clock's solar noon, hours: summer time.
  const SOLAR_NOON: f32 = 13.0;
  /// The sun's illuminance above the atmosphere, lux (the solar illuminance constant).
  const SUN_ILLUMINANCE: f32 = 128_000.0;
  /// The full moon's above the atmosphere, lux.
  const MOON_ILLUMINANCE: f32 = 0.3;
  /// The atmosphere's optical depth at the zenith at about 680, 550 and 440 nm: Rayleigh scattering (Hillaire's
  /// coefficients over an 8 km scale height), a clear day's aerosol (an optical depth of 0.1 at 550 nm, Ångström
  /// exponent 1.3) and ozone (Hillaire's absorption over its 15 km layer).
  const OPTICAL_DEPTH: Vec3 = Vec3::new(
    0.0464 + 0.0759 + 0.0098,
    0.1085 + 0.1 + 0.0282,
    0.2648 + 0.1337 + 0.0013,
  );
  /// The geometric horizon's depression, degrees, under which the sun has set.
  const SUNSET: f32 = -0.833;
  /// The clear sky's illuminance on a level surface by the sun's height (degrees, lux): from a moonless night's
  /// starlight and airglow through the twilights' ends (−18°, −12°, −6°) to a high summer sun's diffuse light.
  const CLEAR_SKY: [(f32, f32); 12] = [
    (-18.0, 0.001),
    (-12.0, 0.008),
    (-6.0, 3.4),
    (-3.0, 60.0),
    (0.0, 400.0),
    (5.0, 3_500.0),
    (10.0, 6_500.0),
    (20.0, 10_500.0),
    (30.0, 13_500.0),
    (45.0, 16_500.0),
    (60.0, 18_000.0),
    (90.0, 19_500.0),
  ];
  /// How much brighter an overcast sky lights than a clear one where the clouds hold the whole sun back, and how much
  /// of it heavy rain leaves.
  const OVERCAST_SKY: f32 = 0.3;
  const RAIN_SKY: f32 = 0.6;
  /// The moonlit sky's illuminance as a share of the moon's.
  const MOONLIT_SKY: f32 = 0.05;
  /// A clear night's moon in the shipped cycles (`w_clear1`'s 0.09), which its cloud factor is read against.
  const CLEAR_MOON: f32 = 0.09;

  /// The world's light under `lighting`, the weather's own sun colour kept in its hue by `tint` (nought the
  /// atmosphere's alone, one the weather's).
  pub fn of(lighting: &RenderLighting, tint: f32) -> Self {
    let clock: f32 = Self::get_sun_elevation(lighting.time_of_day);
    let is_moon: bool = lighting
      .sky
      .sun
      .as_deref()
      .is_some_and(|sprite| sprite.to_ascii_lowercase().starts_with("moon"));
    // How much of its sun or moon the weather lets through: the authored colour's peak, which the shipped cycles hold
    // at one clear, about a quarter clouded and nought before sunrise.
    let peak: f32 = lighting.sun_color.max_element().max(0.0);
    let height: f32 = (-lighting.get_sun_direction().y).clamp(-1.0, 1.0).asin().to_degrees();
    let through: Vec3 = Self::get_transmittance(height);
    let (source, cloud): (f32, f32) = if is_moon {
      (Self::MOON_ILLUMINANCE, (peak / Self::CLEAR_MOON).min(1.0))
    } else if clock > Self::SUNSET {
      (Self::SUN_ILLUMINANCE, peak.min(1.0))
    } else {
      (0.0, 0.0)
    };
    let atmosphere: Vec3 = through * source * cloud;
    let sun: Vec3 = Self::tinted(
      atmosphere,
      authored_colour(lighting.sun_color.max(Vec3::ZERO), true),
      tint,
    );
    let day_sky: f32 = Self::get_clear_sky(clock)
      * if is_moon {
        1.0
      } else {
        1.0 + Self::OVERCAST_SKY * (1.0 - cloud)
      }
      * (1.0 - Self::RAIN_SKY * lighting.rain.map_or(0.0, |rain| rain.density.clamp(0.0, 1.0)));
    let moonlit_sky: f32 = if is_moon {
      Self::MOONLIT_SKY * luminance(atmosphere)
    } else {
      0.0
    };

    Self {
      sun,
      sky: day_sky + moonlit_sky,
    }
  }

  /// The sun's height, degrees, at a time of day (seconds since midnight) by the Zone's latitude and the cycles'
  /// declination.
  pub fn get_sun_elevation(time_of_day: f32) -> f32 {
    let (latitude, declination): (f32, f32) = (Self::LATITUDE.to_radians(), Self::DECLINATION.to_radians());
    let hour_angle: f32 = (15.0 * (time_of_day / 3600.0 - Self::SOLAR_NOON)).to_radians();

    (latitude.sin() * declination.sin() + latitude.cos() * declination.cos() * hour_angle.cos())
      .clamp(-1.0, 1.0)
      .asin()
      .to_degrees()
  }

  /// The share of the light above the atmosphere that reaches the ground through it, by channel, from a source a
  /// height up (degrees): the optical depth along Kasten and Young's air mass (1989); none from under the horizon.
  pub fn get_transmittance(height: f32) -> Vec3 {
    if height <= 0.0 {
      return Vec3::ZERO;
    }

    let air_mass: f32 = 1.0 / (height.to_radians().sin() + 0.50572 * (height + 6.07995).powf(-1.6364));

    (-Self::OPTICAL_DEPTH * air_mass).exp()
  }

  /// The clear sky's illuminance on a level surface, lux, for the sun's height (degrees), between the table's heights
  /// by the logarithm of the light.
  pub fn get_clear_sky(height: f32) -> f32 {
    let table: &[(f32, f32)] = &Self::CLEAR_SKY;
    let (first, last) = (table[0], table[table.len() - 1]);

    if height <= first.0 {
      return first.1;
    }

    if height >= last.0 {
      return last.1;
    }

    let after: usize = table.iter().position(|(at, _)| *at > height).unwrap_or(table.len() - 1);
    let ((low, below), (high, above)) = (table[after - 1], table[after]);
    let share: f32 = (height - low) / (high - low);

    (below.ln() + (above.ln() - below.ln()) * share).exp()
  }

  /// The atmosphere's light given the weather's hue by `tint`, its luminance kept.
  fn tinted(atmosphere: Vec3, authored: Vec3, tint: f32) -> Vec3 {
    let strength: f32 = luminance(atmosphere);
    let tint: f32 = tint.clamp(0.0, 1.0);

    if strength <= 0.0 || tint <= 0.0 || luminance(authored) <= 0.0 {
      return atmosphere;
    }

    let hue = |colour: Vec3| colour / luminance(colour);

    hue(atmosphere).lerp(hue(authored), tint) * strength
  }
}

fn luminance(colour: Vec3) -> f32 {
  colour.dot(Vec3::new(0.2126, 0.7152, 0.0722))
}
