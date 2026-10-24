use xrf_error::XrfResult;

use crate::parts::LevelMapPart;
use crate::state::LevelMapDescription;

#[test]
fn reads_the_map_corners_and_texture_a_level_config_places() -> XrfResult {
  // Zaton's own, as Call of Pripyat ships it.
  let text: &str = "[level_map]\nbound_rect = -600.000000,-615.000000,600.000000,585.000000\n\
                    texture = map\\map_zaton\n;\n;\n[sub_level_map]\n0 = 255\n";

  assert_eq!(
    LevelMapPart::parse(text)?,
    Some(LevelMapDescription {
      min_x: -600.0,
      min_z: -615.0,
      max_x: 600.0,
      max_z: 585.0,
      texture: Some(String::from("map\\map_zaton")),
    })
  );

  Ok(())
}

#[test]
fn reads_no_map_of_a_config_without_its_section() -> XrfResult {
  assert_eq!(LevelMapPart::parse("[map_usage]\nver = 1.0\n")?, None);

  Ok(())
}

#[test]
fn reads_a_map_without_a_texture() -> XrfResult {
  assert_eq!(
    LevelMapPart::parse("[level_map]\nbound_rect = -10, -20, 30, 40\n")?.and_then(|map| map.texture),
    None
  );

  Ok(())
}

#[test]
fn refuses_corners_the_engine_reads_short() {
  // `sscanf("%f,%f,%f,%f")` stops at the suffix, so the engine reads one number of this.
  assert!(LevelMapPart::parse("[level_map]\nbound_rect = -10.0f, -10.0f, 10.0f, 10.0f\n").is_err());
}

#[test]
fn refuses_corners_spanning_no_area() {
  assert!(LevelMapPart::parse("[level_map]\nbound_rect = 10, -10, -10, 10\n").is_err());
}
