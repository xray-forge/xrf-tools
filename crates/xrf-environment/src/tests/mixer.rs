use std::path::PathBuf;

use serde::Serialize;
use xrf_engine_target::XrayEngine;

use super::fixtures::{EnvironmentFixture, extended_keyframe, sun_table, vanilla_keyframe};
use crate::{SunPosition, WeatherDescriptor, WeatherMix, WeatherMixer, WeatherSunSource};

/// A keyframe with some of its keys written otherwise.
fn keyframe(written: String, overrides: &[(&str, &str)]) -> String {
  written
    .lines()
    .map(|line| {
      overrides
        .iter()
        .find(|(key, _)| line.starts_with(&format!("{key} = ")))
        .map_or_else(|| line.to_owned(), |(key, value)| format!("{key} = {value}"))
    })
    .collect::<Vec<_>>()
    .join("\n")
    + "\n\n"
}

fn vanilla_day() -> String {
  [
    keyframe(
      vanilla_keyframe("00:00:00"),
      &[
        ("sky_texture", "sky\\night"),
        ("sun_altitude", "-10"),
        ("sun_longitude", "-5"),
        ("fog_distance", "150"),
        ("fog_density", "0.8"),
        ("hemisphere_color", "0.05, 0.06, 0.1, 1"),
        ("sun_color", "0.02, 0.02, 0.05"),
      ],
    ),
    keyframe(
      vanilla_keyframe("06:00:00"),
      &[
        ("sky_texture", "sky\\dawn"),
        ("sun_altitude", "80"),
        ("sun_longitude", "-8"),
        ("sky_rotation", "20"),
      ],
    ),
    keyframe(
      vanilla_keyframe("12:00:00"),
      &[
        ("sky_texture", "sky\\noon"),
        ("sun_altitude", "-69"),
        ("sun_longitude", "-60"),
        ("far_plane", "900"),
        ("fog_distance", "850"),
        ("fog_density", "0.1"),
        ("sun_color", "0.9, 0.84, 0.7"),
        ("hemisphere_color", "0.47, 0.37, 0.33, 1"),
      ],
    ),
    keyframe(
      vanilla_keyframe("21:00:00"),
      &[
        ("sky_texture", "sky\\dusk"),
        ("sun_altitude", "-150"),
        ("sun_longitude", "-3"),
        ("sky_rotation", "300"),
      ],
    ),
  ]
  .concat()
}

fn extended_day() -> String {
  [
    keyframe(
      extended_keyframe("00:00:00"),
      &[("sky_texture", "sky\\night"), ("fog_distance", "900")],
    ),
    keyframe(
      extended_keyframe("08:00:00"),
      &[
        ("sky_texture", "sky\\morning"),
        ("fog_distance", "600"),
        ("far_plane", "400"),
      ],
    ),
    keyframe(
      extended_keyframe("16:00:00"),
      &[("sky_texture", "sky\\evening"), ("sun_color", "1, 0.8, 0.6")],
    ),
  ]
  .concat()
}

fn read_cycle(fixture: &EnvironmentFixture, engine: XrayEngine) -> (Vec<WeatherDescriptor>, Option<Vec<SunPosition>>) {
  let catalog = fixture.read(engine);

  assert_eq!(catalog.findings, Vec::new());

  let keyframes: Vec<WeatherDescriptor> = catalog
    .find_cycle("test")
    .unwrap()
    .keyframes
    .iter()
    .map(|keyframe| WeatherDescriptor::new(keyframe, engine))
    .collect();

  (
    keyframes,
    catalog.sun_table.as_ref().map(|table| table.list_positions()),
  )
}

fn vanilla_fixture() -> EnvironmentFixture {
  EnvironmentFixture::new().with("environment\\weathers\\test.ltx", &vanilla_day())
}

fn extended_fixture() -> EnvironmentFixture {
  EnvironmentFixture::new()
    .with("environment\\weathers\\test.ltx", &extended_day())
    .with("environment\\sun_positions.ltx", &sun_table())
}

#[test]
fn selects_the_keyframes_around_a_time_and_around_midnight() {
  let (keyframes, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let mixer = WeatherMixer {
    engine: XrayEngine::Vanilla,
    keyframes: &keyframes,
    sun: WeatherSunSource::Authored,
  };

  assert_eq!(mixer.select(0.0), Some([3, 0]));
  assert_eq!(mixer.select(1.0), Some([0, 1]));
  assert_eq!(mixer.select(21_600.0), Some([0, 1]));
  assert_eq!(mixer.select(30_000.0), Some([1, 2]));
  assert_eq!(mixer.select(80_000.0), Some([3, 0]));

  // From nine in the evening to midnight, weighed across it.
  let late: WeatherMix = mixer.mix(81_000.0).unwrap();

  assert!((late.weight - 5_400.0 / 10_800.0).abs() < 1e-6);
  assert_eq!(mixer.mix(21_600.0).unwrap().weight, 1.0);
  // A time outside the day wraps into it.
  assert_eq!(mixer.mix(86_400.0 + 30_000.0), mixer.mix(30_000.0));
}

#[test]
fn mixes_fog_as_the_engine_does() {
  let (keyframes, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let mix: WeatherMix = WeatherMixer {
    engine: XrayEngine::Vanilla,
    keyframes: &keyframes,
    sun: WeatherSunSource::Authored,
  }
  .mix(32_400.0)
  .unwrap();

  // Halfway from six (450 at 0.25) to noon (850 at 0.1).
  assert!((mix.fog_distance - 650.0).abs() < 1e-3);
  assert!((mix.fog_density - 0.175).abs() < 1e-6);
  assert!((mix.fog_near - (1.0 - 0.175) * 0.85 * 650.0).abs() < 1e-3);
  assert!((mix.fog_far - 0.99 * 650.0).abs() < 1e-3);
  assert!(mix.sun_direction[1] < 0.0);
}

#[test]
fn keeps_monolith_fog_inside_its_far_plane() {
  let (keyframes, positions) = read_cycle(&extended_fixture(), XrayEngine::Extended);
  let positions: Vec<SunPosition> = positions.expect("Monolith reads its sun table");
  let mix: WeatherMix = WeatherMixer {
    engine: XrayEngine::Extended,
    keyframes: &keyframes,
    sun: WeatherSunSource::Table(&positions),
  }
  .mix(28_800.0)
  .unwrap();

  assert_eq!(mix.far_plane, 400.0);
  assert_eq!(mix.fog_distance, 390.0);
}

#[test]
fn sets_the_dynamic_sun_below_the_horizon_at_night() {
  let (noon, noon_blend) = WeatherSunSource::dynamic(43_200.0, 0.0);
  let (_, midnight_blend) = WeatherSunSource::dynamic(0.0, 0.0);

  assert!(noon[1] < -0.5);
  assert_eq!(noon_blend, 1.0);
  assert_eq!(midnight_blend, 0.0);
}

#[test]
fn lerps_the_sun_table_by_the_minute() {
  let positions: Vec<SunPosition> = (0..24)
    .map(|hour| SunPosition {
      altitude: hour as f32 * 15.0,
      longitude: -26.0,
    })
    .collect();
  let direction: [f32; 3] = WeatherSunSource::table(&positions, 12.5 * 3600.0);
  let expected: [f32; 3] = WeatherDescriptor::direction_of(187.5f32.to_radians(), (-26f32).to_radians());

  assert!(direction.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-5));
}

/// One cycle mixed through a day, as the renderer's own mixer replays it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoldenCase {
  name: &'static str,
  engine: XrayEngine,
  sun: &'static str,
  keyframes: Vec<WeatherDescriptor>,
  sun_table: Option<Vec<SunPosition>>,
  mixes: Vec<WeatherMix>,
}

/// Times through a day: keyframes exactly, either side of them, around midnight and past the day's end.
const GOLDEN_TIMES: [f32; 14] = [
  0.0, 1.0, 3_600.0, 21_599.0, 21_600.0, 25_000.0, 28_800.0, 43_200.0, 50_000.0, 64_000.0, 75_600.0, 81_000.0,
  86_399.0, 90_000.0,
];

/// Writes the golden vectors the renderer's weather tests replay: `cargo test -p xrf-environment -- --ignored`.
#[test]
#[ignore = "writes the renderer's golden vectors"]
fn writes_the_renderer_golden_vectors() {
  let (vanilla, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let (extended, positions) = read_cycle(&extended_fixture(), XrayEngine::Extended);
  let positions: Vec<SunPosition> = positions.unwrap();
  let case = |name: &'static str, engine: XrayEngine, keyframes: &[WeatherDescriptor], sun: WeatherSunSource| {
    let mixer = WeatherMixer { engine, keyframes, sun };

    GoldenCase {
      engine,
      keyframes: keyframes.to_vec(),
      mixes: GOLDEN_TIMES.iter().map(|time| mixer.mix(*time).unwrap()).collect(),
      name,
      sun: match sun {
        WeatherSunSource::Authored => "authored",
        WeatherSunSource::Dynamic => "dynamic",
        WeatherSunSource::Table(_) => "table",
      },
      sun_table: match sun {
        WeatherSunSource::Table(positions) => Some(positions.to_vec()),
        _ => None,
      },
    }
  };
  let cases: Vec<GoldenCase> = vec![
    case(
      "vanilla, authored sun",
      XrayEngine::Vanilla,
      &vanilla,
      WeatherSunSource::Authored,
    ),
    case(
      "vanilla, dynamic sun",
      XrayEngine::Vanilla,
      &vanilla,
      WeatherSunSource::Dynamic,
    ),
    case(
      "extended, sun table",
      XrayEngine::Extended,
      &extended,
      WeatherSunSource::Table(&positions),
    ),
  ];
  let path: PathBuf =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/xrf-renderer/src/weather/weather-mix.golden.json");

  std::fs::create_dir_all(path.parent().unwrap()).unwrap();
  std::fs::write(&path, serde_json::to_string_pretty(&cases).unwrap() + "\n").unwrap();
}
