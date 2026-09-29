use xrf_engine_target::XrayEngine;

use super::fixtures::{EnvironmentFixture, extended_keyframe, messages, sun_table, vanilla_keyframe};
use crate::{EnvironmentRule, EnvironmentValue, SunPosition, WeatherCycleKind, WeatherDescriptor, WeatherKey};

fn vanilla_cycle(keyframes: &str) -> EnvironmentFixture {
  EnvironmentFixture::new().with("environment\\weathers\\test.ltx", keyframes)
}

#[test]
fn reads_a_clean_cycle_with_nothing_to_say() {
  let fixture: EnvironmentFixture = vanilla_cycle(&(vanilla_keyframe("12:00:00") + &vanilla_keyframe("00:00:00")));
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(catalog.findings, Vec::new());

  let cycle = catalog.find_cycle("test").expect("the cycle is read");

  assert_eq!(cycle.kind, WeatherCycleKind::Cycle);
  // Sorted by time as the engine sorts them, whatever order the file writes them in.
  assert_eq!(
    cycle
      .keyframes
      .iter()
      .map(|keyframe| keyframe.section.name.as_str())
      .collect::<Vec<_>>(),
    vec!["00:00:00", "12:00:00"]
  );
}

#[test]
fn loads_a_keyframe_as_the_engine_holds_it() {
  let fixture: EnvironmentFixture = vanilla_cycle(&(vanilla_keyframe("00:00:00") + &vanilla_keyframe("12:00:00")));
  let catalog = fixture.read(XrayEngine::Vanilla);
  let noon: WeatherDescriptor =
    WeatherDescriptor::new(&catalog.find_cycle("test").unwrap().keyframes[1], catalog.engine);

  assert_eq!(noon.time, 43200);
  assert_eq!(noon.sky_texture_env, "sky\\sky_cube#small");
  assert!((noon.sky_rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
  // The clouds' colour is scaled by half its fifth component, its alpha kept.
  assert_eq!(noon.clouds_color, [0.1, 0.2, 0.3, 0.8]);
  // Rotated as the sky is, where not written.
  assert_eq!(noon.clouds_rotation, noon.sky_rotation);
  assert!((noon.wind_direction - std::f32::consts::PI).abs() < 1e-6);
  // Defaults the keyframe leaves out.
  assert_eq!(noon.water_intensity, 1.0);
  assert_eq!(noon.tree_amplitude, 0.005);
  assert_eq!(noon.thunderbolt_period, 10.0);

  // `setHP(altitude, longitude)`: the altitude turns the heading, the longitude the pitch below the horizon.
  let [x, y, z] = noon.sun_direction.expect("OpenXRay stands the sun by the keyframe");
  let expected = WeatherDescriptor::direction_of((-45f32).to_radians(), (-30f32).to_radians());

  assert!((x - expected[0]).abs() < 1e-6 && (y - expected[1]).abs() < 1e-6 && (z - expected[2]).abs() < 1e-6);
  assert!(y < 0.0);
}

#[test]
fn drops_the_clouds_colour_without_its_multiplier_as_the_engine_does() {
  let keyframe: String = vanilla_keyframe("00:00:00").replace("0.2, 0.4, 0.6, 0.8, 1.0", "0.2, 0.4, 0.6, 0.8");
  let fixture: EnvironmentFixture = vanilla_cycle(&(keyframe + &vanilla_keyframe("12:00:00")));
  let catalog = fixture.read(XrayEngine::Vanilla);
  let midnight = WeatherDescriptor::new(&catalog.find_cycle("test").unwrap().keyframes[0], catalog.engine);

  assert_eq!(catalog.findings, Vec::new());
  assert_eq!(midnight.clouds_color, [0.0, 0.0, 0.0, 0.8]);
}

#[test]
fn says_which_required_key_is_missing() {
  let keyframe: String = vanilla_keyframe("00:00:00").replace("far_plane = 500\n", "");
  let fixture: EnvironmentFixture = vanilla_cycle(&(keyframe + &vanilla_keyframe("12:00:00")));
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(
    messages(&catalog, EnvironmentRule::Engine),
    vec!["Weather [00:00:00] is missing required field [far_plane]"]
  );

  // A viewer still sees the descriptor's constructed value.
  let midnight = WeatherDescriptor::new(&catalog.find_cycle("test").unwrap().keyframes[0], catalog.engine);

  assert_eq!(midnight.far_plane, 400.0);
}

#[test]
fn takes_either_hemisphere_spelling_and_either_sun_form_on_openxray() {
  let keyframe: String = vanilla_keyframe("00:00:00")
    .replace("hemisphere_color", "hemi_color")
    .replace("sun_altitude = -45\n", "")
    .replace("sun_longitude = -30\n", "sun_dir = -30, -45\n");
  let fixture: EnvironmentFixture = vanilla_cycle(&(keyframe + &vanilla_keyframe("12:00:00")));
  let catalog = fixture.read(XrayEngine::Vanilla);
  let midnight = WeatherDescriptor::new(&catalog.find_cycle("test").unwrap().keyframes[0], catalog.engine);

  assert_eq!(catalog.findings, Vec::new());
  assert_eq!(midnight.hemi_color, [0.3, 0.3, 0.3, 1.0]);
  assert!(midnight.is_sun_fixed);
  assert_eq!(
    midnight.sun_direction,
    Some(WeatherDescriptor::direction_of(
      (-45f32).to_radians(),
      (-30f32).to_radians()
    ))
  );
}

#[test]
fn requires_the_sun_angles_on_openxray_without_sun_dir() {
  let keyframe: String = vanilla_keyframe("00:00:00").replace("sun_longitude = -30\n", "");
  let fixture: EnvironmentFixture = vanilla_cycle(&(keyframe + &vanilla_keyframe("12:00:00")));

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Engine),
    vec!["Weather [00:00:00] is missing required field [sun_longitude]"]
  );
}

#[test]
fn keeps_what_the_engine_reads_from_a_malformed_value_and_says_so() {
  let keyframe: String = vanilla_keyframe("00:00:00")
    .replace("fog_density = 0.25", "fog_density = 0.25x")
    .replace("sun_color = 1, 0.9, 0.8", "sun_color = 1, 0.9")
    .replace("rain_color = 0.7, 0.7, 0.7", "rain_color = 0.7, 0.7, 0.7, 0.7");
  let fixture: EnvironmentFixture = vanilla_cycle(&(keyframe + &vanilla_keyframe("12:00:00")));
  let catalog = fixture.read(XrayEngine::Vanilla);
  let section = &catalog.find_cycle("test").unwrap().keyframes[0].section;

  assert_eq!(
    section.get(WeatherKey::FogDensity),
    Some(&EnvironmentValue::Number(0.25))
  );
  // `r_fvector3` starts from zero, so the component it does not find is a zero, not the default.
  assert_eq!(
    section.get_vector::<3>(WeatherKey::SunColor, catalog.engine),
    [1.0, 0.9, 0.0]
  );
  assert_eq!(
    messages(&catalog, EnvironmentRule::Engine),
    vec![
      "Weather [00:00:00] has [sun_color] = '1, 0.9'; the engine reads 2 of 3 components from it and leaves the rest \
       at zero",
    ]
  );
  // `atof` reads the number it starts with, so trailing text is written loosely rather than misread.
  assert_eq!(
    messages(&catalog, EnvironmentRule::Convention),
    vec![
      "Weather [00:00:00] has [fog_density] = '0.25x', whose trailing text the engine ignores, reading 0.25",
      "Weather [00:00:00] has [rain_color] = '0.7, 0.7, 0.7, 0.7' with 4 components; the engine reads the first 3",
    ]
  );
}

#[test]
fn misreads_text_with_no_number_in_it_as_zero() {
  let keyframe: String = vanilla_keyframe("00:00:00").replace("far_plane = 500", "far_plane = far");
  let fixture: EnvironmentFixture = vanilla_cycle(&(keyframe + &vanilla_keyframe("12:00:00")));

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Engine),
    vec!["Weather [00:00:00] has [far_plane] = 'far', which is not a number; the engine reads it as 0"]
  );
}

#[test]
fn warns_of_a_colour_past_the_engines_own_check() {
  let bright: String = vanilla_keyframe("00:00:00").replace("sky_color = 1, 1, 1", "sky_color = 3, 3, 3");
  let fixture: EnvironmentFixture = vanilla_cycle(&(bright + &vanilla_keyframe("12:00:00")));

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Convention),
    vec!["Weather [00:00:00] has [sky_color] outside 0..2, which the engine warns about as invalid"]
  );

  // Monolith checks against five, so Anomaly's brighter skies are not a mistake there.
  let extended: EnvironmentFixture = EnvironmentFixture::new()
    .with(
      "environment\\weathers\\test.ltx",
      &(extended_keyframe("00:00:00") + &extended_keyframe("12:00:00")),
    )
    .with("environment\\sun_positions.ltx", &sun_table());

  assert_eq!(extended.read(XrayEngine::Extended).findings, Vec::new());
}

#[test]
fn needs_two_keyframes_for_a_day_but_not_for_an_effect() {
  let fixture: EnvironmentFixture = vanilla_cycle(&vanilla_keyframe("00:00:00"))
    .with("environment\\weather_effects\\flash.ltx", &vanilla_keyframe("00:00:05"));
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(
    messages(&catalog, EnvironmentRule::Engine),
    vec!["Weather cycle [test] has 1 keyframes, where the engine requires at least two"]
  );
  assert_eq!(
    catalog.find_effect("flash").map(|effect| effect.kind),
    Some(WeatherCycleKind::Effect)
  );
}

#[test]
fn says_what_it_makes_of_a_keyframe_time() {
  let fixture: EnvironmentFixture =
    vanilla_cycle(&(vanilla_keyframe("00:00:00") + &vanilla_keyframe("24:00:00") + &vanilla_keyframe("12:00:00")));

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Engine),
    vec!["Weather [24:00:00] is not a time of day the engine reads, which is HH:MM:SS"]
  );

  // Monolith reads a short name as the hour, which is worth saying but not refused.
  let extended: EnvironmentFixture = EnvironmentFixture::new()
    .with(
      "environment\\weathers\\test.ltx",
      &(extended_keyframe("00:00:00") + &extended_keyframe("12:00")),
    )
    .with("environment\\sun_positions.ltx", &sun_table());

  assert_eq!(
    messages(&extended.read(XrayEngine::Extended), EnvironmentRule::Convention),
    vec!["Weather [12:00] is read as 43200 but is not written HH:MM:SS"]
  );
}

#[test]
fn says_when_two_keyframes_fall_at_one_time() {
  let fixture: EnvironmentFixture =
    vanilla_cycle(&(vanilla_keyframe("00:00:00") + &vanilla_keyframe("12:00:00") + &vanilla_keyframe("12:0:0")));

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Convention),
    vec![
      "Weather [12:0:0] falls at the same time as [12:00:00], so the engine never blends between them",
      "Weather [12:0:0] is read as 43200 but is not written HH:MM:SS",
    ]
  );
}

#[test]
fn reads_an_extended_keyframe_by_monoliths_rules() {
  let fixture: EnvironmentFixture = EnvironmentFixture::new()
    .with(
      "environment\\weathers\\test.ltx",
      &(extended_keyframe("00:00:00") + &extended_keyframe("12:00:00")),
    )
    .with("environment\\sun_positions.ltx", &sun_table());
  let catalog = fixture.read(XrayEngine::Extended);
  let noon = WeatherDescriptor::new(&catalog.find_cycle("test").unwrap().keyframes[1], catalog.engine);

  assert_eq!(catalog.findings, Vec::new());
  // Monolith stands the sun by its table, and reads what OpenXRay does not.
  assert_eq!(noon.sun_direction, None);
  assert_eq!(noon.bloom_threshold, 2.5);
  assert_eq!(noon.tree_amplitude, 0.01);
  assert_eq!(noon.tree_speed, 1.0);
  assert_eq!(
    catalog.sun_table.as_ref().map(|table| table.get_hour(12)),
    Some(SunPosition {
      altitude: 180.0,
      longitude: -26.0
    })
  );

  // The same keyframe on OpenXRay lacks the sun's angles.
  let vanilla = fixture.read(XrayEngine::Vanilla);

  assert!(
    messages(&vanilla, EnvironmentRule::Engine)
      .contains(&"Weather [12:00:00] is missing required field [sun_altitude]")
  );
}

#[test]
fn requires_the_sun_table_on_monolith() {
  let fixture: EnvironmentFixture = EnvironmentFixture::new().with(
    "environment\\weathers\\test.ltx",
    &(extended_keyframe("00:00:00") + &extended_keyframe("12:00:00")),
  );

  assert_eq!(
    messages(&fixture.read(XrayEngine::Extended), EnvironmentRule::Engine),
    vec!["There is no environment\\sun_positions.ltx, which the engine stands the sun by"]
  );
}

#[test]
fn records_where_each_value_came_from_when_asked() {
  let fixture: EnvironmentFixture = vanilla_cycle(&(vanilla_keyframe("00:00:00") + &vanilla_keyframe("12:00:00")));
  let plain = fixture.read(XrayEngine::Vanilla);
  let explained = fixture.read_explained(XrayEngine::Vanilla);

  assert!(plain.cycles[0].keyframes[0].section.origins.is_empty());
  assert!(
    explained.cycles[0].keyframes[0]
      .section
      .origins
      .contains_key("far_plane")
  );
}

#[test]
fn counts_each_texture_the_cycles_and_effects_write_by_the_keyframes_writing_it() {
  let night: String = vanilla_keyframe("00:00:00").replace("sky_texture = sky\\sky_cube", "sky_texture = sky\\night");
  let clear: String = vanilla_keyframe("12:00:00").replace("clouds_texture = sky\\clouds", "clouds_texture =");
  let fixture: EnvironmentFixture =
    vanilla_cycle(&(night + &clear)).with("environment\\weather_effects\\flash.ltx", &vanilla_keyframe("00:00:05"));
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(
    catalog
      .count_texts(WeatherKey::SkyTexture)
      .into_iter()
      .collect::<Vec<_>>(),
    vec![("sky\\night".to_owned(), 1), ("sky\\sky_cube".to_owned(), 2)]
  );
  // A keyframe writing none names no texture.
  assert_eq!(
    catalog
      .count_texts(WeatherKey::CloudsTexture)
      .into_iter()
      .collect::<Vec<_>>(),
    vec![("sky\\clouds".to_owned(), 2)]
  );
}
