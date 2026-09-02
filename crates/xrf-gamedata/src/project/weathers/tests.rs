use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayLogicalPath;

use super::verify_weathers_result::GamedataWeathersVerificationResult;
use super::weather_definitions::WeatherDefinitions;
use super::weather_validator::verify_weather_with_definitions;
use crate::GamedataProject;
use crate::{
  GamedataCheckResult, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationStatus,
};

static NEXT_TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

/// Contents of a `system.ltx` that cannot be parsed, used to exercise legacy fallback failures.
const UNREADABLE_SYSTEM_LTX: &str = "[legacy_duplicate]\n\n[legacy_duplicate]\n";

fn semantic_weather_project(weather: &str) -> (PathBuf, GamedataProject) {
  semantic_weather_project_files(&[("test.ltx", weather)], None)
}

fn semantic_weather_project_with_missing_texture(
  weather: &str,
  missing_texture: Option<&str>,
) -> (PathBuf, GamedataProject) {
  semantic_weather_project_files(&[("test.ltx", weather)], missing_texture)
}

fn semantic_weather_project_files(
  weather_files: &[(&str, &str)],
  missing_texture: Option<&str>,
) -> (PathBuf, GamedataProject) {
  semantic_weather_project_files_with_system(weather_files, missing_texture, "")
}

fn semantic_weather_project_files_with_system(
  weather_files: &[(&str, &str)],
  missing_texture: Option<&str>,
  system_ltx: &str,
) -> (PathBuf, GamedataProject) {
  let unique: u64 = NEXT_TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("weathers/fixture-{unique}"));
  let configs: PathBuf = root.join("configs");

  // A panicking test never reaches its own cleanup below, so the project must not open on what it left.
  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(configs.join("$scheme")).unwrap();
  fs::create_dir_all(configs.join("environment").join("weathers")).unwrap();
  fs::create_dir_all(root.join("textures").join("sky")).unwrap();
  fs::write(configs.join("system.ltx"), system_ltx).unwrap();
  fs::write(
    configs.join("$scheme").join("environment.scheme.ltx"),
    "[$weather]\n$strict = true\nambient = string\nclouds_texture = string\nsky_texture = string\nsun = ?string\nthunderbolt_collection = ?string\n",
  )
  .unwrap();
  fs::write(configs.join("environment").join("ambients.ltx"), "[ambient_ok]\n").unwrap();
  fs::write(configs.join("environment").join("suns.ltx"), "[sun_ok]\n").unwrap();
  fs::write(
    configs.join("environment").join("thunderbolt_collections.ltx"),
    "[bolt_ok]\n\n[bolt_bad]\nmissing_bolt =\n",
  )
  .unwrap();
  fs::write(configs.join("environment").join("thunderbolts.ltx"), "").unwrap();
  for (name, weather) in weather_files {
    fs::write(configs.join("environment").join("weathers").join(name), weather).unwrap();
  }

  for texture in [
    "clouds.dds",
    "first.dds",
    "first#small.dds",
    "second.dds",
    "second#small.dds",
  ] {
    if missing_texture != Some(texture) {
      fs::write(root.join("textures").join("sky").join(texture), "").unwrap();
    }
  }

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();

  (root, project)
}

fn weather_section(time: &str, ambient: &str, sky: &str) -> String {
  format!(
    r#"[{time}]
$scheme = $weather
ambient = {ambient}
ambient_color = 0, 0, 0
clouds_color = 0, 0, 0, 1
clouds_texture = sky\clouds
far_plane = 500
fog_color = 0, 0, 0
fog_density = 0.25
fog_distance = 500
hemisphere_color = 0, 0, 0, 1
rain_color = 1, 1, 1
rain_density = 0
sky_color = 1, 1, 1
sky_rotation = 0
sky_texture = sky\{sky}
sun = sun_ok
sun_altitude = 0
sun_color = 0, 0, 0
sun_longitude = 0
sun_shafts_intensity = 0
thunderbolt_collection = bolt_ok
thunderbolt_duration = 0
thunderbolt_period = 0
water_intensity = 1
wind_direction = 0
wind_velocity = 0
"#
  )
}

#[test]
fn weather_parse_failure_makes_the_check_fail() {
  let valid_weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project_files(
    &[
      ("valid.ltx", &valid_weather),
      (
        "invalid.ltx",
        "[00:00:00]\nvalue = first\n[00:00:00]\nvalue = duplicate\n",
      ),
    ],
    None,
  );
  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 2);
  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_eq!(result.get_failure_message(), "1/2 weather files valid");
}

#[test]
fn out_of_range_weather_time_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("24:00:00", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 1);
  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn noncanonical_weather_time_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("0:0:0", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn weather_requires_at_least_two_time_sections() {
  let weather: String = weather_section("00:00:00", "ambient_ok", "first");
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn missing_ambient_reference_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "missing_ambient", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn missing_sun_reference_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first").replace("sun = sun_ok", "sun = missing_sun"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn missing_thunderbolt_collection_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first").replace(
      "thunderbolt_collection = bolt_ok",
      "thunderbolt_collection = missing_collection",
    ),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn missing_small_sky_texture_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) =
    semantic_weather_project_with_missing_texture(&weather, Some("first#small.dds"));

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn missing_cloud_texture_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) =
    semantic_weather_project_with_missing_texture(&weather, Some("clouds.dds"));

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn missing_required_weather_field_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first").replace("far_plane = 500\n", ""),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert!(
    result
      .findings
      .iter()
      .any(|finding| { finding.message() == "Weather [00:00:00] is missing required field [far_plane]" })
  );
}

#[test]
fn malformed_weather_values_make_the_check_fail() {
  for malformed_section in [
    weather_section("00:00:00", "ambient_ok", "first").replace("fog_density = 0.25", "fog_density = invalid"),
    weather_section("00:00:00", "ambient_ok", "first").replace("fog_color = 0, 0, 0", "fog_color = 0, 0"),
  ] {
    let weather: String = format!(
      "{}{}",
      malformed_section,
      weather_section("12:00:00", "ambient_ok", "second")
    );
    let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

    let result: GamedataWeathersVerificationResult = project
      .verify_weathers(&GamedataProjectVerifyOptions {
        output: xrf_output::OutputOptions::default(),
        ..Default::default()
      })
      .unwrap();

    fs::remove_dir_all(&root).unwrap();

    assert_eq!(result.invalid_weather_files_count, 1);
    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  }
}

#[test]
fn incorrect_weather_scheme_marker_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first").replace("$scheme = $weather", "$scheme = $other"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn negative_weather_distance_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first").replace("far_plane = 500", "far_plane = -1"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn thunderbolt_collection_with_missing_member_makes_the_check_fail() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first")
      .replace("thunderbolt_collection = bolt_ok", "thunderbolt_collection = bolt_bad"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.invalid_weather_files_count, 1);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

#[test]
fn valid_weather_cycle_passes() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 1);
  assert_eq!(result.invalid_weather_files_count, 0);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn primary_weather_definitions_survive_unreadable_legacy_system_fallback() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first"),
    weather_section("12:00:00", "ambient_ok", "second")
  );
  let (root, project): (PathBuf, GamedataProject) =
    semantic_weather_project_files_with_system(&[("test.ltx", &weather)], None, UNREADABLE_SYSTEM_LTX);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 1);
  assert_eq!(result.invalid_weather_files_count, 0);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn missing_primary_names_resolve_from_legacy_system_fallback() {
  let legacy_references: String = weather_section("00:00:00", "ambient_ok", "first")
    .replace("sun = sun_ok", "sun = legacy_sun")
    .replace(
      "thunderbolt_collection = bolt_ok",
      "thunderbolt_collection = legacy_bolt",
    );
  let repeated_legacy_references: String = weather_section("12:00:00", "ambient_ok", "second")
    .replace("sun = sun_ok", "sun = legacy_sun")
    .replace(
      "thunderbolt_collection = bolt_ok",
      "thunderbolt_collection = legacy_bolt",
    );
  let weather: String = format!("{}{}", legacy_references, repeated_legacy_references);
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project_files_with_system(
    &[("test.ltx", &weather)],
    None,
    "[legacy_sun]\n\n[legacy_bolt]\nlegacy_thunderbolt =\n\n[legacy_thunderbolt]\n",
  );

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 1);
  assert_eq!(result.invalid_weather_files_count, 0);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn unreadable_legacy_fallback_is_reported_once_when_missing_names_consult_it() {
  let missing_references: String = weather_section("00:00:00", "ambient_ok", "first")
    .replace("sun = sun_ok", "sun = legacy_sun")
    .replace(
      "thunderbolt_collection = bolt_ok",
      "thunderbolt_collection = legacy_bolt",
    );
  let repeated_missing_references: String = weather_section("12:00:00", "ambient_ok", "second")
    .replace("sun = sun_ok", "sun = legacy_sun")
    .replace(
      "thunderbolt_collection = bolt_ok",
      "thunderbolt_collection = legacy_bolt",
    );
  let weather: String = format!("{}{}", missing_references, repeated_missing_references);
  let (root, project): (PathBuf, GamedataProject) =
    semantic_weather_project_files_with_system(&[("test.ltx", &weather)], None, UNREADABLE_SYSTEM_LTX);
  let definitions: WeatherDefinitions = WeatherDefinitions::read(&project.ltx_project);
  // A logical path now, since the project addresses its files that way and reads them itself.
  let config_path: XrayLogicalPath =
    XrayLogicalPath::new("configs\\environment\\weathers\\test.ltx").expect("valid logical path");
  let mut definition_load_errors: BTreeSet<String> = BTreeSet::new();

  let is_valid: bool = verify_weather_with_definitions(
    &project,
    &GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    },
    &config_path,
    &definitions,
    &mut definition_load_errors,
  )
  .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert!(!is_valid);
  assert_eq!(definition_load_errors.len(), 1);
  assert!(
    definition_load_errors
      .first()
      .is_some_and(|error| error.contains("system.ltx"))
  );
}

#[test]
fn weather_cycle_allows_disabled_sun_and_thunderbolts() {
  let weather: String = format!(
    "{}{}",
    weather_section("00:00:00", "ambient_ok", "first")
      .replace("sun = sun_ok", "sun =")
      .replace("thunderbolt_collection = bolt_ok", "thunderbolt_collection ="),
    weather_section("12:00:00", "ambient_ok", "second")
      .replace("sun = sun_ok", "sun =")
      .replace("thunderbolt_collection = bolt_ok", "thunderbolt_collection =")
  );
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project(&weather);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 1);
  assert_eq!(result.invalid_weather_files_count, 0);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn missing_weather_cycles_make_the_check_fail() {
  let (root, project): (PathBuf, GamedataProject) = semantic_weather_project_files(&[], None);

  let result: GamedataWeathersVerificationResult = project
    .verify_weathers(&GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

  fs::remove_dir_all(&root).unwrap();

  assert_eq!(result.checked_weather_files_count, 0);
  assert_eq!(result.invalid_weather_files_count, 0);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_eq!(result.get_failure_message(), "No weather files found");
}
