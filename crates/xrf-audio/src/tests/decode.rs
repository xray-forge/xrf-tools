use std::sync::Arc;
use std::time::Duration;

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
