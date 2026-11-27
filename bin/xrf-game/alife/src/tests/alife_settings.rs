use std::time::Duration;

use xrf_game_config::{ConfigFile, ConfigIssues};

use crate::settings::alife_settings::AlifeSettings;

#[test]
fn spaces_the_update_as_xrays_scheduler_does() {
  // Call of Pripyat's bounds of 1 ms: 30 + (500 - 30) / 2.
  assert_eq!(
    AlifeSettings::default().get_update_interval(),
    Duration::from_millis(265)
  );

  let wide: AlifeSettings = AlifeSettings {
    schedule_min: 100,
    schedule_max: 1000,
    ..AlifeSettings::default()
  };

  // 100 + (1000 - 100) / 2.
  assert_eq!(wide.get_update_interval(), Duration::from_millis(550));
}

#[test]
fn switches_online_and_offline_either_side_of_the_switch_distance() {
  let settings: AlifeSettings = AlifeSettings::default();

  assert!((settings.get_online_distance() - 135.0).abs() < 1e-4);
  assert!((settings.get_offline_distance() - 165.0).abs() < 1e-4);
}

#[test]
fn reads_alife_from_the_configs_and_says_what_takes_call_of_pripyats() {
  let file: ConfigFile = ConfigFile::parse(
    "alife.ltx",
    "[alife]\nswitch_distance = 750\nobjects_per_update = 20\nswitch_factor = lots\nschedule_min = 1\n",
  )
  .expect("the configs read");
  let mut issues: ConfigIssues = ConfigIssues::default();
  let settings: AlifeSettings = AlifeSettings::read(&mut file.read_section("alife", &mut issues).expect("[alife]"));

  assert!((settings.switch_distance - 750.0).abs() < 1e-4);
  assert_eq!(settings.objects_per_update, 20);
  assert!((settings.switch_factor - 0.1).abs() < 1e-6);
  assert_eq!(settings.schedule_min, 1);
  assert_eq!(
    issues.iter().map(ToString::to_string).collect::<Vec<_>>(),
    vec![
      "alife.ltx [alife] switch_factor: 'lots' is no number",
      "alife.ltx [alife] schedule_max: is missing"
    ]
  );
}
