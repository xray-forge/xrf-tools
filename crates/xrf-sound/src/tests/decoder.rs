use xrf_error::XrfResult;

use crate::fixtures::VorbisNoiseFixture;
use crate::{SoundFile, VorbisDecoder};

/// Every frame of `bytes` decoded from the start, interleaved, checking each packet starts where the one before it
/// stopped.
fn decode_whole(bytes: &[u8]) -> XrfResult<Vec<f32>> {
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes)?;
  let channels: usize = usize::from(decoder.get_channels());
  let mut samples: Vec<f32> = Vec::new();
  let mut packet: Vec<f32> = Vec::new();

  while let Some(first_frame) = decoder.decode_next(&mut packet)? {
    assert_eq!(
      first_frame as usize,
      samples.len() / channels,
      "packets follow one another"
    );
    samples.extend_from_slice(&packet);
  }

  Ok(samples)
}

#[test]
fn decodes_a_sound_to_the_length_its_last_page_places() -> XrfResult {
  for channels in [1, 2] {
    let bytes: Vec<u8> = VorbisNoiseFixture::sound(channels, 400);
    let file: SoundFile = SoundFile::read_strictly_from_bytes(&bytes)?;
    let decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes.as_slice())?;
    let samples: Vec<f32> = decode_whole(&bytes)?;

    assert_eq!(decoder.get_channels(), channels);
    assert_eq!(decoder.get_sample_rate(), VorbisNoiseFixture::SAMPLE_RATE);
    assert_eq!(decoder.get_samples_count(), file.samples_count);
    assert_eq!(samples.len() as u64, file.samples_count * u64::from(channels));
    assert!(samples.iter().filter(|sample| **sample != 0.0).count() > samples.len() / 2);
  }

  Ok(())
}

#[test]
fn seeks_forward_and_back_to_the_frames_a_straight_decode_gives() -> XrfResult {
  let bytes: Vec<u8> = VorbisNoiseFixture::sound(2, 600);
  let whole: Vec<f32> = decode_whole(&bytes)?;
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes.as_slice())?;
  let samples_count: u64 = decoder.get_samples_count();
  let mut packet: Vec<f32> = Vec::new();
  let mut frames: Vec<u64> = vec![
    0,
    1,
    127,
    128,
    129,
    samples_count / 2,
    samples_count - 1,
    5000,
    3,
    samples_count - 2,
  ];
  let mut random: u64 = 0x2545_F491_4F6C_DD1D;

  for _ in 0..300 {
    random ^= random << 13;
    random ^= random >> 7;
    random ^= random << 17;
    frames.push(random % samples_count);
  }

  for frame in frames {
    decoder.seek(frame)?;

    let first_frame: u64 = decoder
      .decode_next(&mut packet)?
      .expect("a frame inside the sound decodes");
    let mut decoded: Vec<f32> = packet.clone();

    // A fixture page ends at most six packets of 1024 frames after the one before it.
    assert!(
      first_frame <= frame,
      "a seek to {frame} lands at {first_frame}, past it"
    );
    assert!(
      frame - first_frame < 2 * 6 * 1024,
      "a seek to {frame} lands far before it, at {first_frame}"
    );

    while decoded.len() < 2 * (frame - first_frame + 4096) as usize
      && let Some(next) = decoder.decode_next(&mut packet)?
    {
      assert_eq!(
        next,
        first_frame + decoded.len() as u64 / 2,
        "packets follow one another after a seek"
      );
      decoded.extend_from_slice(&packet);
    }

    let start: usize = 2 * first_frame as usize;

    assert_eq!(
      decoded,
      whole[start..(start + decoded.len()).min(whole.len())],
      "a seek to {frame}"
    );
  }

  Ok(())
}

#[test]
fn a_seek_to_the_end_leaves_nothing_to_decode() -> XrfResult {
  let bytes: Vec<u8> = VorbisNoiseFixture::sound(1, 50);
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes.as_slice())?;
  let mut packet: Vec<f32> = vec![1.0];

  for frame in [decoder.get_samples_count(), decoder.get_samples_count() + 10_000] {
    decoder.seek(frame)?;

    while let Some(first_frame) = decoder.decode_next(&mut packet)? {
      assert!(
        first_frame + (packet.len() as u64) <= frame,
        "nothing at or past {frame} decodes"
      );
    }

    assert!(packet.is_empty());
  }

  Ok(())
}

#[test]
fn refuses_bytes_that_are_no_vorbis_sound() {
  for bytes in [&b"OggS this is no sound at all"[..], &[]] {
    assert!(VorbisDecoder::open(bytes).is_err());
  }
}
