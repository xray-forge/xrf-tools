use std::time::Duration;

use xrf_ltx::Ltx;

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
fn reads_alife_from_the_configs_and_keeps_the_defaults_for_what_they_lack() {
  let ltx: Ltx = Ltx::read_from_str("[alife]\nswitch_distance = 750\nobjects_per_update = 20\nswitch_factor = lots\n")
    .expect("the configs read");
  let settings: AlifeSettings = AlifeSettings::read_ltx(&ltx);

  assert!((settings.switch_distance - 750.0).abs() < 1e-4);
  assert_eq!(settings.objects_per_update, 20);
  assert!((settings.switch_factor - 0.1).abs() < 1e-6);
  assert_eq!(settings.schedule_min, 1);
  assert_eq!(
    AlifeSettings::read_ltx(&Ltx::read_from_str("[other]\n").expect("reads")),
    AlifeSettings::default()
  );
}
