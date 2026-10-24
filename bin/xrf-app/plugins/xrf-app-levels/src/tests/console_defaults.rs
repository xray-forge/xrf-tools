use crate::console::LevelConsoleDefaults;

#[test]
fn reads_the_lighting_anomaly_ships_among_its_other_commands() {
  let text: &str = "bind forward kW\nr2_sun_lumscale 2.0\nr2_sun_lumscale_amb 1.\n; r2_sun_lumscale_hemi 9\n\
    r2_tonemap on\nr2_tonemap_middlegray 1.5\nr2_tonemap_lowlum 0.5\nr__color_grading (0.5, 0.25, 0)\n\
    r2_ls_bloom_threshold 0.\nr2_ls_bloom_kernel_g 1.\nr2_ls_bloom_kernel_scale 0.05\n";

  assert_eq!(
    LevelConsoleDefaults::parse(text),
    LevelConsoleDefaults {
      is_tonemapped: Some(true),
      tonemap_middle_gray: Some(1.5),
      tonemap_low_luminance: Some(0.5),
      sun_scale: Some(2.0),
      ambient_scale: Some(1.0),
      color_grading: Some([0.5, 0.25, 0.0]),
      bloom_threshold: Some(0.0),
      bloom_radius: Some(1.0),
      bloom_strength: Some(0.05),
      ..LevelConsoleDefaults::default()
    }
  );
}

#[test]
fn leaves_to_the_engine_whatever_no_command_sets_or_it_cannot_read() {
  let defaults: LevelConsoleDefaults = LevelConsoleDefaults::parse("r2_tonemap maybe\nr__gamma bright\n");

  assert_eq!(defaults.is_tonemapped, None);
  assert_eq!(defaults.image_gamma, None);
}

// A vanilla installation that has been run has a `user.ltx` and no shipped defaults: its console ships none.
#[test]
fn says_nothing_shipped_for_a_user_console_alone() {
  let console: LevelConsoleDefaults =
    LevelConsoleDefaults::parse("r2_tonemap on\n").over(LevelConsoleDefaults::default());

  assert!(!console.is_shipped);
  assert_eq!(console.is_tonemapped, Some(true));
}
