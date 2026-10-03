use std::path::PathBuf;

use serde::Serialize;
use xrf_engine_target::XrayEngine;

use super::fixtures::{EnvironmentFixture, extended_keyframe, sun_table, vanilla_keyframe};
use crate::{
  SunPosition, WeatherDescriptor, WeatherMix, WeatherMixPoint, WeatherMixer, WeatherModifier, WeatherSunSource,
};

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
        ("thunderbolt_collection", ""),
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
        ("thunderbolt_period", "20"),
        ("rain_density", "0.6"),
        ("rain_color", "0.4, 0.45, 0.5"),
        ("wind_velocity", "12"),
        ("wind_direction", "45"),
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

pub(super) fn read_cycle(
  fixture: &EnvironmentFixture,
  engine: XrayEngine,
) -> (Vec<WeatherDescriptor>, Option<Vec<SunPosition>>) {
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

/// A time of day, seen from the origin.
fn at(time: f32) -> WeatherMixPoint {
  WeatherMixPoint {
    time,
    view: [0.0, 0.0, 0.0],
  }
}

pub(super) fn vanilla_fixture() -> EnvironmentFixture {
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
    sun: WeatherSunSource::Authored,
    modifiers: &[],
  };

  assert_eq!(WeatherMixer::select(&keyframes, 0.0), Some([3, 0]));
  assert_eq!(WeatherMixer::select(&keyframes, 1.0), Some([0, 1]));
  assert_eq!(WeatherMixer::select(&keyframes, 21_600.0), Some([0, 1]));
  assert_eq!(WeatherMixer::select(&keyframes, 30_000.0), Some([1, 2]));
  assert_eq!(WeatherMixer::select(&keyframes, 80_000.0), Some([3, 0]));

  // From nine in the evening to midnight, weighed across it.
  let late: WeatherMix = mixer.mix(&keyframes, at(81_000.0)).unwrap();

  assert!((late.weight - 5_400.0 / 10_800.0).abs() < 1e-6);
  assert_eq!(late.between, [75_600.0, 0.0]);
  assert_eq!(mixer.mix(&keyframes, at(21_600.0)).unwrap().weight, 1.0);
  // A time outside the day wraps into it.
  assert_eq!(
    mixer.mix(&keyframes, at(86_400.0 + 30_000.0)),
    mixer.mix(&keyframes, at(30_000.0))
  );
}

#[test]
fn mixes_fog_as_the_engine_does() {
  let (keyframes, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let mix: WeatherMix = WeatherMixer {
    engine: XrayEngine::Vanilla,
    sun: WeatherSunSource::Authored,
    modifiers: &[],
  }
  .mix(&keyframes, at(32_400.0))
  .unwrap();

  // Halfway from six (450 at 0.25) to noon (850 at 0.1).
  assert!((mix.fog_distance - 650.0).abs() < 1e-3);
  assert!((mix.fog_density - 0.175).abs() < 1e-6);
  assert!((mix.fog_near - (1.0 - 0.175) * 0.85 * 650.0).abs() < 1e-3);
  assert!((mix.fog_far - 0.99 * 650.0).abs() < 1e-3);
  assert!(mix.sun_direction[1] < 0.0);
}

// `m_fSunShaftsIntensity` is lerped as every scalar is.
#[test]
fn mixes_the_sun_shafts_as_the_engine_does() {
  let (mut keyframes, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);

  keyframes[1].sun_shafts_intensity = 0.1;
  keyframes[2].sun_shafts_intensity = 0.3;

  let mix: WeatherMix = WeatherMixer {
    engine: XrayEngine::Vanilla,
    sun: WeatherSunSource::Authored,
    modifiers: &[],
  }
  .mix(&keyframes, at(32_400.0))
  .unwrap();

  assert!((mix.sun_shafts_intensity - 0.2).abs() < 1e-6);
}

// `CEnvDescriptorMixer::lerp`: the collection is the nearer keyframe's, the timings blended.
#[test]
fn mixes_thunderbolts_as_the_engine_does() {
  let (keyframes, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let mixer = WeatherMixer {
    engine: XrayEngine::Vanilla,
    sun: WeatherSunSource::Authored,
    modifiers: &[],
  };
  // A quarter from six, which strikes with nothing, to noon; then three quarters.
  let early: WeatherMix = mixer.mix(&keyframes, at(27_000.0)).unwrap();
  let late: WeatherMix = mixer.mix(&keyframes, at(37_800.0)).unwrap();
  // Halfway from nine in the evening to midnight.
  let night: WeatherMix = mixer.mix(&keyframes, at(81_000.0)).unwrap();

  assert_eq!(early.thunderbolt_collection, None);
  assert!((early.thunderbolt_period - 2.5).abs() < 1e-5);
  assert_eq!(late.thunderbolt_collection.as_deref(), Some("bolts"));
  assert!((late.thunderbolt_duration - 0.375).abs() < 1e-5);
  assert!((night.thunderbolt_period - 15.0).abs() < 1e-4);
}

#[test]
fn keeps_monolith_fog_inside_its_far_plane() {
  let (keyframes, positions) = read_cycle(&extended_fixture(), XrayEngine::Extended);
  let positions: Vec<SunPosition> = positions.expect("Monolith reads its sun table");
  let mix: WeatherMix = WeatherMixer {
    engine: XrayEngine::Extended,
    sun: WeatherSunSource::Table(&positions),
    modifiers: &[],
  }
  .mix(&keyframes, at(28_800.0))
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

/// Two volumes, one adding to everything and one to the fog alone, overlapping.
fn modifiers() -> Vec<WeatherModifier> {
  vec![
    WeatherModifier {
      ambient: [0.2, 0.1, 0.0],
      far_plane: 300.0,
      flags: WeatherModifier::ALL,
      fog_color: [0.1, 0.2, 0.3],
      fog_density: 0.5,
      hemi_color: [0.3, 0.3, 0.1],
      position: [0.0, 0.0, 0.0],
      power: 1.0,
      radius: 50.0,
      sky_color: [0.5, 0.4, 0.3],
    },
    WeatherModifier {
      ambient: [1.0, 1.0, 1.0],
      far_plane: 1000.0,
      flags: WeatherModifier::FOG_COLOR | WeatherModifier::FOG_DENSITY,
      fog_color: [0.9, 0.1, 0.1],
      fog_density: 1.0,
      hemi_color: [1.0, 1.0, 1.0],
      position: [30.0, 0.0, 0.0],
      power: 0.5,
      radius: 40.0,
      sky_color: [1.0, 1.0, 1.0],
    },
  ]
}

#[test]
fn weighs_the_modifiers_reaching_the_view() {
  let (keyframes, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let modifiers: Vec<WeatherModifier> = modifiers();
  let mixer = WeatherMixer {
    engine: XrayEngine::Vanilla,
    modifiers: &modifiers,
    sun: WeatherSunSource::Authored,
  };
  let point = |view: [f32; 3]| mixer.mix(&keyframes, WeatherMixPoint { time: 43_200.0, view }).unwrap();
  let plain: WeatherMix = WeatherMixer {
    modifiers: &[],
    ..mixer
  }
  .mix(&keyframes, at(43_200.0))
  .unwrap();

  // Out of every reach, nothing changes.
  assert_eq!(point([200.0, 0.0, 0.0]).far_plane, 900.0);
  assert_eq!(point([200.0, 0.0, 0.0]).fog_density, plain.fog_density);

  // At the first's centre it adds all of itself, the second an eighth of its power, and the environment keeps what
  // the power of both leaves it.
  let centre: WeatherMix = point([0.0, 0.0, 0.0]);
  let scale: f32 = 1.0 / (1.0 + 1.0 + 0.125);

  assert!((centre.far_plane - (900.0 + 300.0) * scale).abs() < 1e-3);
  assert!((centre.fog_density - (0.1 + 0.5 + 0.125) * scale).abs() < 1e-6);
  assert!((centre.fog_near - (1.0 - centre.fog_density) * 0.85 * centre.fog_distance).abs() < 1e-3);
  assert_eq!(centre.hemi_color[3], plain.hemi_color[3]);

  // Where only the second reaches, only the fog is touched.
  let fogged: WeatherMix = point([60.0, 0.0, 0.0]);

  assert_eq!(fogged.far_plane, plain.far_plane);
  assert_eq!(fogged.sky_color, plain.sky_color);
  assert_ne!(fogged.fog_color, plain.fog_color);
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
  modifiers: Vec<WeatherModifier>,
  mixes: Vec<WeatherMix>,
}

/// Times through a day: keyframes exactly, either side of them, around midnight and past the day's end.
const GOLDEN_TIMES: [f32; 14] = [
  0.0, 1.0, 3_600.0, 21_599.0, 21_600.0, 25_000.0, 28_800.0, 43_200.0, 50_000.0, 64_000.0, 75_600.0, 81_000.0,
  86_399.0, 90_000.0,
];

/// Views across the golden modifiers: both centres, where they overlap, and out of reach.
const GOLDEN_VIEWS: [[f32; 3]; 4] = [[0.0, 0.0, 0.0], [20.0, 5.0, -3.0], [30.0, 0.0, 0.0], [100.0, 0.0, 0.0]];

/// Writes the golden vectors the renderer's weather tests replay: `cargo test -p xrf-environment -- --ignored`.
#[test]
#[ignore = "writes the renderer's golden vectors"]
fn writes_the_renderer_golden_vectors() {
  let (vanilla, _) = read_cycle(&vanilla_fixture(), XrayEngine::Vanilla);
  let (extended, positions) = read_cycle(&extended_fixture(), XrayEngine::Extended);
  let positions: Vec<SunPosition> = positions.unwrap();
  let modifiers: Vec<WeatherModifier> = modifiers();
  let case =
    |name: &'static str, keyframes: &[WeatherDescriptor], mixer: WeatherMixer, views: &[[f32; 3]]| GoldenCase {
      engine: mixer.engine,
      keyframes: keyframes.to_vec(),
      mixes: views
        .iter()
        .flat_map(|view| {
          GOLDEN_TIMES.iter().map(|time| WeatherMixPoint {
            time: *time,
            view: *view,
          })
        })
        .map(|point| mixer.mix(keyframes, point).unwrap())
        .collect(),
      modifiers: mixer.modifiers.to_vec(),
      name,
      sun: match mixer.sun {
        WeatherSunSource::Authored => "authored",
        WeatherSunSource::Dynamic => "dynamic",
        WeatherSunSource::Table(_) => "table",
      },
      sun_table: match mixer.sun {
        WeatherSunSource::Table(positions) => Some(positions.to_vec()),
        _ => None,
      },
    };
  let origin: &[[f32; 3]] = &GOLDEN_VIEWS[..1];
  let cases: Vec<GoldenCase> = vec![
    case(
      "vanilla, authored sun",
      &vanilla,
      WeatherMixer {
        engine: XrayEngine::Vanilla,
        modifiers: &[],
        sun: WeatherSunSource::Authored,
      },
      origin,
    ),
    case(
      "vanilla, dynamic sun",
      &vanilla,
      WeatherMixer {
        engine: XrayEngine::Vanilla,
        modifiers: &[],
        sun: WeatherSunSource::Dynamic,
      },
      origin,
    ),
    case(
      "extended, sun table",
      &extended,
      WeatherMixer {
        engine: XrayEngine::Extended,
        modifiers: &[],
        sun: WeatherSunSource::Table(&positions),
      },
      origin,
    ),
    case(
      "vanilla, modified",
      &vanilla,
      WeatherMixer {
        engine: XrayEngine::Vanilla,
        modifiers: &modifiers,
        sun: WeatherSunSource::Authored,
      },
      &GOLDEN_VIEWS,
    ),
    case(
      "extended, modified",
      &extended,
      WeatherMixer {
        engine: XrayEngine::Extended,
        modifiers: &modifiers,
        sun: WeatherSunSource::Table(&positions),
      },
      &GOLDEN_VIEWS,
    ),
  ];
  let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/weather-mix.golden.json");

  std::fs::create_dir_all(path.parent().unwrap()).unwrap();
  std::fs::write(&path, serde_json::to_string_pretty(&cases).unwrap() + "\n").unwrap();
}
