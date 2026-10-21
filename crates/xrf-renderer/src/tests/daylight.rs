use glam::Vec3;

use crate::lighting::daylight::Daylight;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::sun_direction::to_renderer_sun_direction;

fn luminance(colour: Vec3) -> f32 {
  colour.dot(Vec3::new(0.2126, 0.7152, 0.0722))
}

/// A weather at a time of day with its sun or moon a height up, its sprite and colour.
fn weather(hours: f32, height: f32, sprite: &str, colour: f32) -> RenderLighting {
  let mut lighting: RenderLighting = RenderLighting {
    sun_direction: to_renderer_sun_direction(0.0, -height),
    sun_color: Vec3::splat(colour),
    time_of_day: hours * 3600.0,
    ..RenderLighting::default()
  };

  lighting.sky.sun = Some(sprite.to_owned());
  lighting
}

// The clock stands the sun where the shipped cycles do: 53.4° up at 13:00, about 6° at 06:30 and 19:30 (`w_clear1`
// authors 6.2°), and sets it between the cycles' last sunset and first night keyframes.
#[test]
fn the_clock_stands_the_sun_where_the_cycles_do() {
  assert!((Daylight::get_sun_elevation(13.0 * 3600.0) - 53.4).abs() < 0.2);
  assert!((Daylight::get_sun_elevation(6.5 * 3600.0) - 6.2).abs() < 1.0);
  assert!((Daylight::get_sun_elevation(19.5 * 3600.0) - 6.2).abs() < 1.0);
  assert!(Daylight::get_sun_elevation(5.5 * 3600.0) < -0.833);
  assert!(Daylight::get_sun_elevation(6.0 * 3600.0) > -0.833);
  assert!(Daylight::get_sun_elevation(1.0 * 3600.0) < -12.0);
}

// The sun through a clear atmosphere: about 100 000 lx from the zenith, a little warm; a few tens of thousands and deep
// orange six degrees up, its blue nearly gone; nothing from under the horizon.
#[test]
fn a_clear_atmosphere_dims_and_reddens_a_low_sun() {
  let zenith: Vec3 = Daylight::get_transmittance(90.0) * 128_000.0;
  let low: Vec3 = Daylight::get_transmittance(6.0) * 128_000.0;

  assert!((95_000.0..110_000.0).contains(&luminance(zenith)), "{zenith}");
  assert!(zenith.x > zenith.y && zenith.y > zenith.z);
  assert!((15_000.0..40_000.0).contains(&luminance(low)), "{low}");
  assert!(low.x > 5.0 * low.z, "{low}");
  assert_eq!(Daylight::get_transmittance(-1.0), Vec3::ZERO);
}

// The clear sky's light on a level surface falls through the twilights by the logarithm of the light, from a few
// hundred lux at sunset to a moonless night's thousandth, and stands at its table's values.
#[test]
fn the_clear_sky_falls_through_the_twilights() {
  assert_eq!(Daylight::get_clear_sky(-30.0), 0.001);
  assert!((Daylight::get_clear_sky(0.0) - 400.0).abs() < 1e-2);
  assert!((Daylight::get_clear_sky(-9.0) - (3.4_f32 * 0.008).sqrt()).abs() < 1e-3);

  let mut last: f32 = 0.0;

  for height in -20..=90 {
    let sky: f32 = Daylight::get_clear_sky(height as f32);

    assert!(sky >= last, "{height}");
    last = sky;
  }
}

// The weather's sun lets the clear atmosphere's through as far as its colour's peak, its sky brightening as the clouds
// hold the sun back and darkening in rain; a moon's sprite makes its light the moon's, a fraction of a lux; a night
// without one has no sun.
#[test]
fn the_weather_keeps_its_clouds_and_its_moon() {
  let clear: Daylight = Daylight::of(&weather(13.0, 53.4, "clear_day", 1.0), 0.0);
  let clouded: Daylight = Daylight::of(&weather(13.0, 53.4, "cloudy_light_day", 0.25), 0.0);
  let mut raining: RenderLighting = weather(13.0, 53.4, "cloudy_light_day", 0.0);

  raining.rain = Some(RenderRainfall {
    color: Vec3::ONE,
    density: 1.0,
  });

  let rain: Daylight = Daylight::of(&raining, 0.0);
  let moon: Daylight = Daylight::of(&weather(1.0, 30.0, "moon_full_clear", 0.09), 0.0);
  let dark: Daylight = Daylight::of(&weather(1.0, 30.0, "none", 0.0), 0.0);

  assert!((luminance(clouded.sun) / luminance(clear.sun) - 0.25).abs() < 1e-4);
  assert!(clouded.sky > clear.sky && rain.sky < clear.sky);
  assert!((0.1..0.3).contains(&luminance(moon.sun)), "{}", moon.sun);
  assert!(moon.sky > dark.sky && moon.sky < 0.1);
  assert_eq!(dark.sun, Vec3::ZERO);
}

// The tint takes the sun's hue towards the weather's own and keeps the atmosphere's strength.
#[test]
fn the_tint_keeps_the_strength_and_takes_the_weathers_hue() {
  let mut lighting: RenderLighting = weather(13.0, 53.4, "clear_day", 1.0);

  lighting.sun_color = Vec3::new(1.0, 0.8, 0.65);

  let physical: Daylight = Daylight::of(&lighting, 0.0);
  let authored: Daylight = Daylight::of(&lighting, 1.0);

  assert!((luminance(authored.sun) - luminance(physical.sun)).abs() / luminance(physical.sun) < 1e-4);
  assert!(authored.sun.z / authored.sun.x < physical.sun.z / physical.sun.x);
}
