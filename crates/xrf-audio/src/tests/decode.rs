use std::sync::Arc;
use std::time::Duration;

use xrf_error::XrfResult;
use xrf_sound::SoundFile;
use xrf_sound::fixtures::VorbisNoiseFixture;

use crate::tests::fixtures::{RATE, ramp, wav};
use crate::{DecodedSound, SoundChannels, SoundDecoder, SoundPeaks};

fn get_frame(sample: f32) -> i64 {
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
    assert_eq!(get_frame(packet[0]), frame.cast_signed(), "seek to {frame}");
  }

  for end in [10_000, 20_000] {
    decoder.seek(end).expect("a seek to or past the end lands there");
    assert!(!decoder.decode_next(&mut packet).expect("the end decodes to nothing"));
    assert!(packet.is_empty());
  }

  decoder.seek(3).expect("a seek back from the end");
  assert!(decoder.decode_next(&mut packet).expect("the sound decodes again"));
  assert_eq!(get_frame(packet[0]), 3);
}

// A float WAV can hold any 32 bits, and one that is no number would otherwise reach the device and the reverb.
#[test]
fn decodes_a_sample_that_is_no_number_as_silence() {
  let samples: [f32; 4] = [0.5, f32::NAN, f32::INFINITY, -0.25];
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(b"RIFF");
  bytes.extend_from_slice(&(36 + 16u32).to_le_bytes());
  bytes.extend_from_slice(b"WAVEfmt ");
  bytes.extend_from_slice(&16u32.to_le_bytes());
  // IEEE float, one channel, 32 bits a sample.
  bytes.extend_from_slice(&3u16.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&RATE.to_le_bytes());
  bytes.extend_from_slice(&(RATE * 4).to_le_bytes());
  bytes.extend_from_slice(&4u16.to_le_bytes());
  bytes.extend_from_slice(&32u16.to_le_bytes());
  bytes.extend_from_slice(b"data");
  bytes.extend_from_slice(&16u32.to_le_bytes());

  for sample in samples {
    bytes.extend_from_slice(&sample.to_le_bytes());
  }

  let sound: DecodedSound = DecodedSound::decode(bytes.into()).expect("the sound decodes");

  assert_eq!(sound.get_samples(), &[0.5, 0.0, 0.0, -0.25]);
}

#[test]
fn decodes_an_ogg_sound_to_the_length_the_engine_counts() -> XrfResult {
  let bytes: Arc<[u8]> = VorbisNoiseFixture::sound(2, 300).into();
  let file: SoundFile = SoundFile::read_from_bytes(&bytes)?;
  let decoder: SoundDecoder = SoundDecoder::open(Arc::clone(&bytes))?;
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
  let whole: DecodedSound = DecodedSound::decode(Arc::clone(&bytes))?;
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
    assert!(DecodedSound::decode(Arc::clone(&bytes)).is_err());
    assert!(SoundDecoder::open(Arc::clone(&bytes)).is_err());
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

#[test]
fn refuses_a_waveform_of_more_stretches_than_it_reads_in() {
  let bytes: Arc<[u8]> = wav(RATE, 1, &[0; 16]);

  assert!(SoundPeaks::read(Arc::clone(&bytes), SoundPeaks::MAX_COUNT).is_ok());
  assert!(SoundPeaks::read(bytes, SoundPeaks::MAX_COUNT + 1).is_err());
}
