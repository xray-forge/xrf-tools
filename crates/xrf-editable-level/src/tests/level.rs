use xrf_error::XrfResult;

use crate::tests::fixtures::{GUID, level_text};
use crate::{EditableBuildParams, EditableLevelFile, EditableLevelOptions, EditableLevelValue};

#[test]
fn reads_a_scenes_main_file_and_writes_it_back_as_it_was() -> XrfResult {
  let text: String = level_text(&GUID)?;
  let level: EditableLevelFile = EditableLevelFile::read_from_bytes(text.as_bytes())?;

  assert_eq!(level.get_version()?, EditableLevelFile::VERSION);
  assert_eq!(level.get_guid()?, GUID);
  assert_eq!(
    level.get_level_options()?,
    EditableLevelOptions {
      level_path: "labx8".into(),
      level_prefix: "labx8".into(),
      build_options: String::new(),
      map_version: "1.0".into(),
      game_type: 1,
      light_hemi_quality: 3,
      light_sun_quality: 0,
    }
  );
  assert_eq!(level.get_build_params()?.weld_distance, 0.005);
  // The editor wrote -80.052984, which no f32 prints as with `%f` (the nearest print as -80.052986 and -80.052979), so
  // a typed read is close, and only the kept text writes back exactly.
  assert!((level.get_camera_position()?.x - (-80.052_98)).abs() < 1e-4);
  assert_eq!(
    EditableLevelValue::format_vector3(&level.get_camera_position()?),
    "-80.052986, -6.285084, 75.509567"
  );
  assert_eq!(level.write_to_bytes()?, text.as_bytes());

  Ok(())
}

#[test]
fn writes_build_parameters_and_options_in_the_editors_layout() -> XrfResult {
  let mut level: EditableLevelFile = EditableLevelFile::read_from_bytes(level_text(&GUID)?.as_bytes())?;
  let params: EditableBuildParams = EditableBuildParams {
    weld_distance: 0.0,
    ..level.get_build_params()?
  };
  let options: EditableLevelOptions = EditableLevelOptions {
    build_options: "-f".into(),
    ..level.get_level_options()?
  };

  level.set_build_params(&params);
  level.set_level_options(&options);

  assert_eq!(level.ini.get("build_params", "weld_distance"), Some("0.000000"));
  assert_eq!(level.ini.get("level_options", "bop"), Some("\"-f\""));
  assert_eq!(
    level.ini.get("build_params", "reserved_0"),
    Some("18011438074424197120000000.000000")
  );

  let restored: EditableLevelFile = EditableLevelFile::read_from_bytes(&level.write_to_bytes()?)?;

  assert_eq!(restored.get_build_params()?, params);
  assert_eq!(restored.get_level_options()?, options);

  Ok(())
}

#[test]
fn refuses_a_scene_the_editor_would_not_open() -> XrfResult {
  let text: String = level_text(&GUID)?;
  let older: String = text.replace(
    "value                            = 5",
    "value                            = 4",
  );
  let without_guid: String = text.replace("guid_g0", "guid_x0");

  assert!(EditableLevelFile::read_from_bytes(older.as_bytes()).is_err());
  assert!(EditableLevelFile::read_from_bytes(without_guid.as_bytes()).is_err());

  Ok(())
}
