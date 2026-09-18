use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::level_fog_vol_file::LevelFogVolFile;
use crate::tests::fixtures::floats;

/// A 4x4 transform, as the engine blits one, translating to a readable place.
fn matrix() -> Vec<u8> {
  floats(&[
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 2.0, -5.0, 1.0,
  ])
}

/// A fog body: a profile line, a transform, then its obstacles.
fn volume_bytes(profile: &str, terminator: &str, obstacles: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = profile.as_bytes().to_vec();

  bytes.extend_from_slice(terminator.as_bytes());
  bytes.extend(matrix());
  bytes.extend_from_slice(&(obstacles as u32).to_le_bytes());

  for _ in 0..obstacles {
    bytes.extend(matrix());
  }

  bytes
}

/// A whole file: a version, a count, then the bodies.
fn file_bytes(volumes: &[(&str, &str, usize)]) -> Vec<u8> {
  let mut bytes: Vec<u8> = 3u16.to_le_bytes().to_vec();

  bytes.extend_from_slice(&(volumes.len() as u32).to_le_bytes());

  for (profile, terminator, obstacles) in volumes {
    bytes.extend(volume_bytes(profile, terminator, *obstacles));
  }

  bytes
}

#[test]
fn a_fog_body_names_the_config_its_simulation_comes_from() -> XrfResult {
  let file: LevelFogVolFile =
    LevelFogVolFile::read_from_bytes::<XRayByteOrder>(file_bytes(&[("environment\\fog\\area_02.ltx", "\r\n", 2)]))?;

  assert_eq!(file.version, LevelFogVolFile::CURRENT_VERSION);
  assert_eq!(file.volumes.len(), 1);
  assert_eq!(file.volumes[0].profile, "environment\\fog\\area_02.ltx");
  assert_eq!(file.volumes[0].obstacles.len(), 2);
  assert_eq!(file.get_obstacles_count(), 2);
  assert_eq!(file.volumes[0].transform.get_translation().x, 10.0);

  Ok(())
}

#[test]
fn a_profile_is_a_line_rather_than_a_terminated_string() -> XrfResult {
  // `dx113DFluidData::Load` reads it with `r_string`, which stops at the first CR or LF. Every shipped body ends
  // with a CRLF, and normalising that would cost byte-identical repack.
  let file: LevelFogVolFile =
    LevelFogVolFile::read_from_bytes::<XRayByteOrder>(file_bytes(&[("environment\\fog\\area_02.ltx", "\r\n", 0)]))?;

  assert_eq!(file.volumes[0].terminator, "\r\n");

  Ok(())
}

#[test]
fn a_fog_body_list_is_written_back_byte_for_byte() -> XrfResult {
  for volumes in [
    vec![],
    vec![("environment\\fog\\area_01.ltx", "\r\n", 0usize)],
    vec![
      ("environment\\fog\\area_01.ltx", "\r\n", 3usize),
      ("environment\\fog\\area_02.ltx", "\r\n", 1),
    ],
  ] {
    let bytes: Vec<u8> = file_bytes(&volumes);
    let file: LevelFogVolFile = LevelFogVolFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    file.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, bytes, "{} volumes did not round trip", volumes.len());
  }

  Ok(())
}

#[test]
fn a_file_declaring_no_bodies_is_the_ordinary_case() -> XrfResult {
  // 116 of the 117 shipped files are exactly six bytes: a version and a zero count.
  let bytes: Vec<u8> = file_bytes(&[]);
  let file: LevelFogVolFile = LevelFogVolFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(bytes.len(), 6);
  assert!(file.volumes.is_empty());

  Ok(())
}

#[test]
fn a_version_the_renderer_refuses_is_refused_here_too() {
  let mut bytes: Vec<u8> = file_bytes(&[]);

  bytes[0..2].copy_from_slice(&2u16.to_le_bytes());

  assert!(LevelFogVolFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());
}
