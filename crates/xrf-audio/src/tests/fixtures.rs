use std::f32::consts::TAU;
use std::sync::Arc;

use crate::{
  AudioBus, AudioEngine, AudioEngineParts, AudioEngineSettings, AudioPlay, AudioVoiceId, DecodedSound, SoundChannels,
  VoiceSource,
};

/// The rate tests render and write their sounds at.
pub(crate) const RATE: u32 = 48_000;

/// A sine of `frequency` at `amplitude`, `frames` long at `rate`, the same in every channel.
pub(crate) fn sine(rate: u32, frames: usize, frequency: f32, amplitude: f32, channels: SoundChannels) -> DecodedSound {
  let samples: Vec<f32> = (0..frames)
    .flat_map(|frame| {
      let sample: f32 = amplitude * (TAU * frequency * frame as f32 / rate as f32).sin();

      std::iter::repeat_n(sample, channels.get_count())
    })
    .collect();

  DecodedSound::new(rate, channels, samples)
}

/// A 16-bit PCM WAV file of `samples`, interleaved over `channels`.
pub(crate) fn wav(rate: u32, channels: u16, samples: &[i16]) -> Arc<[u8]> {
  let data_length: u32 = (samples.len() * 2) as u32;
  let block_align: u16 = channels * 2;
  let mut bytes: Vec<u8> = Vec::with_capacity(44 + samples.len() * 2);

  bytes.extend_from_slice(b"RIFF");
  bytes.extend_from_slice(&(36 + data_length).to_le_bytes());
  bytes.extend_from_slice(b"WAVEfmt ");
  bytes.extend_from_slice(&16u32.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&channels.to_le_bytes());
  bytes.extend_from_slice(&rate.to_le_bytes());
  bytes.extend_from_slice(&(rate * u32::from(block_align)).to_le_bytes());
  bytes.extend_from_slice(&block_align.to_le_bytes());
  bytes.extend_from_slice(&16u16.to_le_bytes());
  bytes.extend_from_slice(b"data");
  bytes.extend_from_slice(&data_length.to_le_bytes());

  for sample in samples {
    bytes.extend_from_slice(&sample.to_le_bytes());
  }

  bytes.into()
}

/// A mono ramp of `frames` samples, each its own index, so a frame read back names where it came from.
pub(crate) fn ramp(frames: usize) -> Vec<i16> {
  (0..frames).map(|frame| frame as i16).collect()
}

/// `sound` from its start, over and over.
pub(crate) fn looped(sound: &Arc<DecodedSound>) -> VoiceSource {
  VoiceSource::Decoded {
    sound: sound.clone(),
    start_frame: 0,
    is_looped: true,
  }
}

/// `sound` from its start, once.
pub(crate) fn once(sound: &Arc<DecodedSound>) -> VoiceSource {
  VoiceSource::Decoded {
    sound: sound.clone(),
    start_frame: 0,
    is_looped: false,
  }
}

/// Plays `count` looping voices of `sound` over every bus, every tenth audible and the rest at zero gain.
pub(crate) fn play_voices(engine: &mut AudioEngine, sound: &Arc<DecodedSound>, count: usize) -> Vec<AudioVoiceId> {
  (0..count)
    .map(|index| {
      engine
        .play(AudioPlay {
          gain: if index % 10 == 0 { 0.01 } else { 0.0 },
          ..AudioPlay::new(looped(sound), AudioBus::ALL[index % AudioBus::COUNT])
        })
        .expect("a voice starts")
    })
    .collect()
}

/// An engine of `voices_count` voices.
pub(crate) fn engine(voices_count: usize) -> AudioEngineParts {
  AudioEngineParts::new(AudioEngineSettings {
    voices_count,
    ..AudioEngineSettings::default()
  })
}

/// The root mean square of the left channel of interleaved stereo `samples`.
pub(crate) fn rms_left(samples: &[f32]) -> f32 {
  let lefts: Vec<f32> = samples.iter().step_by(2).copied().collect();

  (lefts.iter().map(|sample| sample * sample).sum::<f32>() / lefts.len() as f32).sqrt()
}

/// How often the left channel of interleaved stereo `samples` crosses zero upwards, per second at `rate`.
pub(crate) fn frequency_left(samples: &[f32], rate: u32) -> f32 {
  let lefts: Vec<f32> = samples.iter().step_by(2).copied().collect();
  let crossings: usize = lefts.windows(2).filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0).count();

  crossings as f32 * rate as f32 / lefts.len() as f32
}
