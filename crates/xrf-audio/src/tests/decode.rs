use std::sync::Arc;
use std::time::Duration;

use xrf_error::XrfResult;
use xrf_sound::SoundFile;
use xrf_sound::fixtures::VorbisNoiseFixture;

use crate::tests::fixtures::{RATE, ramp, wav};
use crate::{DecodedSound, SoundChannels, SoundDecoder, SoundPeaks};

/// A ramp sample read back as the frame it was written at.
fn to_frame(sample: f32) -> i64 {
  (sample * 32768.0).round() as i64
}

#[test]
fn decodes_a_whole_sound_to_its_samples() {
  let sound: DecodedSound = DecodedSound::decode(wav(RATE, 2, &[0, 16384, -16384, 32767])).expect("the sound decodes");

  assert_eq!(sound.get_sample_rate(), RATE);
  assert_eq!(sound.get_channels(), SoundChannels::Stereo);
  assert_eq!(sound.get_frames_count(), 2);
  assert_eq!(sound.get_samples(), &[0.0, 0.5, -0.5, 32767.0 / 32768.0]);
}

#[test]
fn seeks_to_the_exact_frame_asked_for() {
  let mut decoder: SoundDecoder = SoundDecoder::open(wav(RATE, 1, &ramp(10_000))).expect("the sound opens");
  let mut packet: Vec<f32> = Vec::new();

  for frame in [0, 1, 4095, 4096, 7777, 9999] {
    decoder.seek(frame).expect("the decoder seeks");
    assert!(decoder.decode_next(&mut packet).expect("the sound decodes"));
    assert_eq!(to_frame(packet[0]), frame as i64, "seek to {frame}");
  }

  for end in [10_000, 20_000] {
    decoder.seek(end).expect("a seek to or past the end lands there");
    assert!(!decoder.decode_next(&mut packet).expect("the end decodes to nothing"));
    assert!(packet.is_empty());
  }

  decoder.seek(3).expect("a seek back from the end");
  assert!(decoder.decode_next(&mut packet).expect("the sound decodes again"));
  assert_eq!(to_frame(packet[0]), 3);
}

#[test]
fn decodes_an_ogg_sound_to_the_length_the_engine_counts() -> XrfResult {
  let bytes: Arc<[u8]> = VorbisNoiseFixture::sound(2, 300).into();
  let file: SoundFile = SoundFile::read_from_bytes(&bytes)?;
  let decoder: SoundDecoder = SoundDecoder::open(bytes.clone())?;
  let sound: DecodedSound = DecodedSound::decode(bytes)?;

  assert_eq!(decoder.get_sample_rate(), VorbisNoiseFixture::SAMPLE_RATE);
  assert_eq!(decoder.get_channels(), SoundChannels::Stereo);
  assert_eq!(decoder.get_frames_count(), file.samples_count);
  assert_eq!(sound.get_frames_count() as u64, file.samples_count);

  Ok(())
}

#[test]
fn seeks_an_ogg_sound_forward_and_back_to_the_frames_a_whole_decode_gives() -> XrfResult {
  let bytes: Arc<[u8]> = VorbisNoiseFixture::sound(2, 500).into();
  let whole: DecodedSound = DecodedSound::decode(bytes.clone())?;
  let mut decoder: SoundDecoder = SoundDecoder::open(bytes)?;
  let frames_count: u64 = decoder.get_frames_count();
  let mut packet: Vec<f32> = Vec::new();
  let forward: Vec<u64> = (0..40).map(|step| step * frames_count / 40 + step % 7).collect();
  let backward: Vec<u64> = forward.iter().rev().map(|frame| frame + 333).collect();

  for frame in forward.into_iter().chain(backward).chain([frames_count - 1, 0, 1]) {
    let start: usize = 2 * frame as usize;
    let mut decoded: Vec<f32> = Vec::new();

    decoder.seek(frame)?;

    while decoded.len() < 2 * 3000 && decoder.decode_next(&mut packet)? {
      decoded.extend_from_slice(&packet);
    }

    assert!(!decoded.is_empty(), "a seek to {frame} decodes");
    assert_eq!(
      decoded,
      whole.get_samples()[start..(start + decoded.len()).min(whole.get_samples().len())],
      "a seek to {frame}"
    );
  }

  for end in [frames_count, frames_count + 50_000] {
    decoder.seek(end)?;
    assert!(!decoder.decode_next(&mut packet)?, "a seek to {end} decodes nothing");
  }

  Ok(())
}

#[test]
fn refuses_what_it_cannot_play() {
  let three_channels: Arc<[u8]> = wav(RATE, 3, &[0; 30]);
  let garbage: Arc<[u8]> = Arc::from(&b"OggS this is no sound at all"[..]);

  for bytes in [three_channels, garbage, Arc::from(&[][..])] {
    assert!(DecodedSound::decode(bytes.clone()).is_err());
    assert!(SoundDecoder::open(bytes.clone()).is_err());
    assert!(SoundPeaks::read(bytes, 16).is_err());
  }
}

#[test]
fn reads_the_loudest_sample_of_each_stretch() {
  let mut samples: Vec<i16> = vec![0; 4000];

  samples[100] = 16384;
  samples[1500] = -32768;
  samples[3999] = 8192;

  let peaks: SoundPeaks = SoundPeaks::read(wav(4000, 1, &samples), 4).expect("the peaks read");

  assert_eq!(peaks.peaks, vec![0.5, 1.0, 0.0, 0.25]);
  assert_eq!(peaks.duration, Duration::from_secs(1));
}

#[test]
fn reads_one_peak_a_frame_from_a_sound_shorter_than_the_count() {
  let peaks: SoundPeaks = SoundPeaks::read(wav(RATE, 2, &[16384, 0, 0, -8192]), 100).expect("the peaks read");

  assert_eq!(peaks.peaks, vec![0.5, 0.25]);
}
