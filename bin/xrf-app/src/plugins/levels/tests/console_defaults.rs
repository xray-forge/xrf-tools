use crate::plugins::levels::console_defaults::parse_console_defaults;
use crate::plugins::levels::state::LevelConsoleDefaults;

#[test]
fn reads_the_lighting_anomaly_ships_among_its_other_commands() {
  let text: &str = "bind forward kW\nr2_sun_lumscale 2.0\nr2_sun_lumscale_amb 1.\n; r2_sun_lumscale_hemi 9\n\
    r2_tonemap on\nr2_tonemap_middlegray 1.5\nr2_tonemap_lowlum 0.5\nr__color_grading (0.5, 0.25, 0)\n";

  assert_eq!(
    parse_console_defaults(text),
    LevelConsoleDefaults {
      is_shipped: true,
      is_tonemapped: Some(true),
      tonemap_middle_gray: Some(1.5),
      tonemap_low_luminance: Some(0.5),
      sun_scale: Some(2.0),
      ambient_scale: Some(1.0),
      color_grading: Some([0.5, 0.25, 0.0]),
      ..LevelConsoleDefaults::default()
    }
  );
}

#[test]
fn leaves_to_the_engine_whatever_no_command_sets_or_it_cannot_read() {
  let defaults: LevelConsoleDefaults = parse_console_defaults("r2_tonemap maybe\nr__gamma bright\n");

  assert!(defaults.is_shipped);
  assert_eq!(defaults.is_tonemapped, None);
  assert_eq!(defaults.image_gamma, None);
}
