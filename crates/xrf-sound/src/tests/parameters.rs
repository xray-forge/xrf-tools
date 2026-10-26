use xrf_error::XrfResult;

use crate::tests::fixtures::xray_comment;
use crate::{XRaySoundCommentVersion, XRaySoundGameType, XRaySoundParameters};

#[test]
fn reads_a_version_three_block() {
  assert_eq!(
    XRaySoundParameters::read(&xray_comment(1.0, 50.0, 0.75, 0x8020_0000, 30.0)),
    Ok(Some((
      XRaySoundCommentVersion::V3,
      XRaySoundParameters {
        min_distance: 1.0,
        max_distance: 50.0,
        base_volume: 0.75,
        game_type: XRaySoundGameType::WeaponShooting,
        max_ai_distance: 30.0,
      }
    )))
  );
}

#[test]
fn a_version_one_block_takes_the_base_volume_and_ai_distance_the_engine_gives_it() {
  let comment: Vec<u8> = [
    1u32.to_le_bytes(),
    2.0f32.to_le_bytes(),
    60.0f32.to_le_bytes(),
    0u32.to_le_bytes(),
  ]
  .concat();
  let (_, parameters) = XRaySoundParameters::read(&comment)
    .expect("a version 1 block reads")
    .expect("version 1 is one the engine reads");

  assert_eq!(parameters.base_volume, 1.0);
  assert_eq!(parameters.max_ai_distance, 60.0);
}

#[test]
fn keeps_a_game_type_the_sdk_has_no_name_for() -> XrfResult {
  let comment: Vec<u8> = xray_comment(1.0, 50.0, 1.0, 0x8000_0000, 50.0);
  let (_, parameters) = XRaySoundParameters::read(&comment)
    .expect("the block reads")
    .expect("version 3 is one the engine reads");

  assert_eq!(parameters.game_type, XRaySoundGameType::Unknown(0x8000_0000));
  assert_eq!(parameters.write()?, comment);

  Ok(())
}

#[test]
fn refuses_distances_the_engine_does_not_load_a_sound_with() {
  assert_eq!(
    XRaySoundParameters::read(&xray_comment(1.0, 0.0, 1.0, 0, 0.0)),
    Err(String::from(
      "X-Ray sound maximum and maximum AI distances must be at least 0.1"
    ))
  );
}

#[test]
fn an_unknown_version_is_no_block_the_engine_reads() {
  let mut comment: Vec<u8> = xray_comment(1.0, 50.0, 1.0, 0, 50.0);

  comment[0..4].copy_from_slice(&4u32.to_le_bytes());

  assert_eq!(XRaySoundParameters::read(&comment), Ok(None));
}

#[test]
fn refuses_a_truncated_block_of_a_version_the_engine_reads() {
  assert_eq!(
    XRaySoundParameters::read(&3u32.to_le_bytes()),
    Err(String::from("Vorbis packet ends before minimum distance"))
  );
}

#[test]
fn clamps_to_the_ranges_the_sdk_editor_holds() {
  let mut parameters: XRaySoundParameters = XRaySoundParameters {
    min_distance: 0.0,
    max_distance: 5000.0,
    base_volume: 3.0,
    game_type: XRaySoundGameType::Undefined,
    max_ai_distance: 2000.0,
  };

  parameters.clamp();

  assert_eq!(parameters.min_distance, 0.01);
  assert_eq!(parameters.max_distance, 1000.0);
  assert_eq!(parameters.base_volume, 2.0);
  assert_eq!(parameters.max_ai_distance, 1000.0);

  parameters.max_distance = 100.0;
  parameters.clamp();

  assert_eq!(
    parameters.max_ai_distance, 100.0,
    "the AI hears no further than the sound carries"
  );
}

#[test]
fn names_game_types_as_the_sdk_does() {
  assert_eq!(XRaySoundGameType::from(0x2000_8000), XRaySoundGameType::NpcStep);
  assert_eq!(XRaySoundGameType::NpcStep.label(), "NPC step");
  assert_eq!(XRaySoundGameType::Unknown(0x8000_0000).label(), "0x80000000");

  for game_type in XRaySoundGameType::NAMED {
    assert_eq!(XRaySoundGameType::from(u32::from(game_type)), game_type);
  }
}
