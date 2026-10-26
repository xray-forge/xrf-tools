use std::time::Duration;

use xrf_error::XrfResult;

use crate::tests::fixtures::{SAMPLES_COUNT, audio_packets, comment, identification, sound, sound_of, xray_comment};
use crate::{SoundFile, SoundMetadata, XRaySoundCommentVersion, XRaySoundGameType, XRaySoundParameters};

#[test]
fn reads_what_the_engine_loads_of_a_sound() -> XrfResult {
  let file: SoundFile = SoundFile::read_from_bytes(&sound(&[xray_comment(2.0, 80.0, 0.5, 0x0800_0080, 40.0)]))?;

  assert_eq!(file.channels, 2);
  assert_eq!(file.sample_rate, 44_100);
  assert_eq!(file.samples_count, SAMPLES_COUNT);
  assert_eq!(file.get_duration(), Duration::from_nanos(68_027_210));
  assert_eq!(
    file.metadata,
    SoundMetadata::XRay {
      version: XRaySoundCommentVersion::V3,
      parameters: XRaySoundParameters {
        min_distance: 2.0,
        max_distance: 80.0,
        base_volume: 0.5,
        game_type: XRaySoundGameType::WorldAmbient,
        max_ai_distance: 40.0,
      },
    }
  );

  Ok(())
}

#[test]
fn a_sound_without_comments_plays_with_the_engine_defaults() -> XrfResult {
  let file: SoundFile = SoundFile::read_from_bytes(&sound(&[]))?;

  assert_eq!(file.metadata, SoundMetadata::EngineDefaults);
  assert_eq!(file.metadata.get_parameters(), XRaySoundParameters::default());

  Ok(())
}

#[test]
fn a_first_comment_that_is_no_xray_block_leaves_the_engine_defaults() -> XrfResult {
  let file: SoundFile = SoundFile::read_from_bytes(&sound(&[b"TITLE=shot".to_vec()]))?;

  assert_eq!(file.metadata, SoundMetadata::EngineDefaults);

  Ok(())
}

#[test]
fn refuses_headers_libvorbis_does_not_open() {
  // `_vorbis_unpack_comment` and `_vorbis_unpack_info` both end on a set framing bit, and the identification packet's
  // block sizes must be 64 to 8192 samples, the short no longer than the long.
  let unframed: Vec<u8> = sound_of(identification(0xB8), comment(&[], 0));
  let inverted_blocks: Vec<u8> = sound_of(identification(0x8B), comment(&[], 1));

  assert!(SoundFile::read_from_bytes(&unframed).is_err());
  assert!(SoundFile::read_from_bytes(&inverted_blocks).is_err());
}

#[test]
fn refuses_a_comment_header_shorter_than_its_comments() {
  let mut header: Vec<u8> = comment(&[b"TITLE=shot".to_vec()], 1);

  header.truncate(header.len() - 4);

  assert!(SoundFile::read_from_bytes(&sound_of(identification(0xB8), header)).is_err());
}

#[test]
fn writes_parameters_into_a_sound_keeping_its_audio_and_other_comments() -> XrfResult {
  let original: Vec<u8> = sound(&[xray_comment(1.0, 50.0, 1.0, 0, 50.0), b"TITLE=shot".to_vec()]);
  let parameters: XRaySoundParameters = XRaySoundParameters {
    min_distance: 3.0,
    max_distance: 120.0,
    base_volume: 0.8,
    game_type: XRaySoundGameType::WeaponShooting,
    max_ai_distance: 90.0,
  };
  let written: Vec<u8> = SoundFile::write_parameters(&original, &parameters)?;
  let file: SoundFile = SoundFile::read_from_bytes(&written)?;

  assert_eq!(file.metadata.get_parameters(), parameters);
  assert_eq!(file.samples_count, SAMPLES_COUNT);
  assert_eq!(audio_packets(&written), audio_packets(&original));
  assert_eq!(
    written,
    SoundFile::write_parameters(&sound(&[parameters.write()?, b"TITLE=shot".to_vec()]), &parameters)?,
    "the X-Ray block is replaced rather than stacked, so the title stays second"
  );

  Ok(())
}

#[test]
fn writes_parameters_ahead_of_a_first_comment_the_engine_does_not_read() -> XrfResult {
  let parameters: XRaySoundParameters = XRaySoundParameters::default();
  let written: Vec<u8> = SoundFile::write_parameters(&sound(&[b"TITLE=shot".to_vec()]), &parameters)?;

  assert_eq!(
    written,
    SoundFile::write_parameters(&sound(&[parameters.write()?, b"TITLE=shot".to_vec()]), &parameters)?
  );

  Ok(())
}

#[test]
fn refuses_to_write_parameters_the_engine_does_not_load() {
  let parameters: XRaySoundParameters = XRaySoundParameters {
    max_distance: 0.0,
    ..XRaySoundParameters::default()
  };

  assert!(SoundFile::write_parameters(&sound(&[]), &parameters).is_err());
}
