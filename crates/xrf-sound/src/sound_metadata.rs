use crate::xray_sound_comment_version::XRaySoundCommentVersion;
use crate::xray_sound_parameters::XRaySoundParameters;

/// Metadata that controls how X-Ray configures a sound source.
#[derive(Clone, Debug, PartialEq)]
pub enum SoundMetadata {
  /// The engine uses its built-in sound-source defaults when no recognized X-Ray comment exists.
  EngineDefaults,
  XRay {
    version: XRaySoundCommentVersion,
    parameters: XRaySoundParameters,
  },
}

/// Read the binary X-Ray sound parameters stored in the first Vorbis comment.
pub fn read_sound_metadata(comment_packet: &[u8]) -> Result<SoundMetadata, String> {
  let metadata: SoundMetadata = first_vorbis_comment(&comment_packet[7..])?
    .map(|comment| parse_xray_comment(&comment))
    .transpose()?
    .flatten()
    .map(|(version, parameters)| SoundMetadata::XRay { version, parameters })
    .unwrap_or(SoundMetadata::EngineDefaults);

  Ok(metadata)
}

fn first_vorbis_comment(packet: &[u8]) -> Result<Option<Vec<u8>>, String> {
  let mut offset: usize = 0;

  let vendor_length: usize = read_u32(packet, &mut offset, "vendor length")? as usize;

  read_bytes(packet, &mut offset, vendor_length, "vendor string")?;

  let comments_count: u32 = read_u32(packet, &mut offset, "comments count")?;

  if comments_count == 0 {
    return Ok(None);
  }

  let comment_length: usize = read_u32(packet, &mut offset, "first comment length")? as usize;

  Ok(Some(
    read_bytes(packet, &mut offset, comment_length, "first comment")?.to_vec(),
  ))
}

fn parse_xray_comment(comment: &[u8]) -> Result<Option<(XRaySoundCommentVersion, XRaySoundParameters)>, String> {
  let mut offset: usize = 0;

  let version: u32 = read_u32(comment, &mut offset, "X-Ray comment version")?;
  let comment_version: XRaySoundCommentVersion = match version {
    0x0001 => XRaySoundCommentVersion::V1,
    0x0002 => XRaySoundCommentVersion::V2,
    0x0003 => XRaySoundCommentVersion::V3,
    _ => return Ok(None),
  };

  let min_distance: f32 = read_f32(comment, &mut offset, "minimum distance")?;
  let max_distance: f32 = read_f32(comment, &mut offset, "maximum distance")?;

  let (base_volume, game_type, max_ai_distance): (f32, u32, f32) = match comment_version {
    XRaySoundCommentVersion::V1 => {
      let game_type: u32 = read_u32(comment, &mut offset, "game type")?;
      (1.0, game_type, max_distance)
    }
    XRaySoundCommentVersion::V2 => {
      let base_volume: f32 = read_f32(comment, &mut offset, "base volume")?;
      let game_type: u32 = read_u32(comment, &mut offset, "game type")?;
      (base_volume, game_type, max_distance)
    }
    XRaySoundCommentVersion::V3 => {
      let base_volume: f32 = read_f32(comment, &mut offset, "base volume")?;
      let game_type: u32 = read_u32(comment, &mut offset, "game type")?;
      let max_ai_distance: f32 = read_f32(comment, &mut offset, "maximum AI distance")?;
      (base_volume, game_type, max_ai_distance)
    }
  };

  let parameters: XRaySoundParameters = XRaySoundParameters {
    min_distance,
    max_distance,
    base_volume,
    game_type,
    max_ai_distance,
  };

  if !parameters.min_distance.is_finite()
    || !parameters.max_distance.is_finite()
    || !parameters.base_volume.is_finite()
    || !parameters.max_ai_distance.is_finite()
  {
    return Err(String::from("X-Ray sound parameters must be finite numbers"));
  }

  if parameters.max_distance < 0.1 || parameters.max_ai_distance < 0.1 {
    return Err(String::from(
      "X-Ray sound maximum and maximum AI distances must be at least 0.1",
    ));
  }

  Ok(Some((comment_version, parameters)))
}

fn read_u32(bytes: &[u8], offset: &mut usize, field: &str) -> Result<u32, String> {
  let value: &[u8] = read_bytes(bytes, offset, 4, field)?;

  Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn read_f32(bytes: &[u8], offset: &mut usize, field: &str) -> Result<f32, String> {
  let value: &[u8] = read_bytes(bytes, offset, 4, field)?;

  Ok(f32::from_le_bytes(value.try_into().unwrap()))
}

fn read_bytes<'a>(bytes: &'a [u8], offset: &mut usize, length: usize, field: &str) -> Result<&'a [u8], String> {
  let end: usize = offset
    .checked_add(length)
    .ok_or_else(|| format!("Vorbis {field} length overflows the packet"))?;
  let value: &[u8] = bytes
    .get(*offset..end)
    .ok_or_else(|| format!("Vorbis packet ends before {field}"))?;

  *offset = end;

  Ok(value)
}

#[cfg(test)]
mod tests {
  use super::{XRaySoundCommentVersion, XRaySoundParameters, first_vorbis_comment, parse_xray_comment};

  fn xray_comment_v3(
    min_distance: f32,
    max_distance: f32,
    base_volume: f32,
    game_type: u32,
    max_ai_distance: f32,
  ) -> Vec<u8> {
    let mut comment: Vec<u8> = Vec::new();
    comment.extend(3u32.to_le_bytes());
    comment.extend(min_distance.to_le_bytes());
    comment.extend(max_distance.to_le_bytes());
    comment.extend(base_volume.to_le_bytes());
    comment.extend(game_type.to_le_bytes());
    comment.extend(max_ai_distance.to_le_bytes());
    comment
  }

  #[test]
  fn parses_valid_xray_v3_comment() {
    assert_eq!(
      parse_xray_comment(&xray_comment_v3(1.0, 50.0, 0.75, 7, 30.0)).unwrap(),
      Some((
        XRaySoundCommentVersion::V3,
        XRaySoundParameters {
          min_distance: 1.0,
          max_distance: 50.0,
          base_volume: 0.75,
          game_type: 7,
          max_ai_distance: 30.0,
        }
      ))
    );
  }

  #[test]
  fn rejects_xray_comment_with_invalid_distances() {
    let error: String = parse_xray_comment(&xray_comment_v3(1.0, 0.0, 1.0, 0, 0.0))
      .expect_err("Expected invalid sound distances to be rejected");

    assert_eq!(
      error,
      "X-Ray sound maximum and maximum AI distances must be at least 0.1"
    );
  }

  #[test]
  fn accepts_unknown_xray_comment_version_as_engine_defaults() {
    let mut comment: Vec<u8> = xray_comment_v3(1.0, 50.0, 1.0, 0, 50.0);
    comment[0..4].copy_from_slice(&4u32.to_le_bytes());

    assert_eq!(parse_xray_comment(&comment).unwrap(), None);
  }

  #[test]
  fn rejects_truncated_recognized_xray_comment() {
    let comment: Vec<u8> = 3u32.to_le_bytes().to_vec();

    assert_eq!(
      parse_xray_comment(&comment).unwrap_err(),
      "Vorbis packet ends before minimum distance"
    );
  }

  #[test]
  fn accepts_empty_vorbis_comments_as_engine_defaults() {
    let comment_packet: Vec<u8> = [0u32.to_le_bytes(), 0u32.to_le_bytes()].concat();

    assert_eq!(first_vorbis_comment(&comment_packet).unwrap(), None);
  }
}
