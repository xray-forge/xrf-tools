//! A recording's sound brought from a device's rate AAC does not take to 48 kHz: in time, in tune, and nothing above
//! 24 kHz folded under it.

use std::f64::consts::TAU;

use crate::viewer::recording::writer::RecordResampler;

/// Interleaved stereo of `frames` at `rate`, a sine of `frequency` at half scale in both channels.
fn sine(rate: u32, frames: usize, frequency: f64) -> Vec<f32> {
  (0..frames)
    .flat_map(|frame| {
      let sample: f32 = ((TAU * frequency * frame as f64 / f64::from(rate)).sin() * 0.5) as f32;

      [sample, sample]
    })
    .collect()
}

/// `input` at `rate` resampled, handed over a tenth of a second at a time as a tap's blocks come.
fn resample(rate: u32, input: &[f32]) -> Vec<f32> {
  let mut resampler: RecordResampler = RecordResampler::for_rate(rate).expect("a rate AAC does not take");
  let mut output: Vec<f32> = Vec::new();

  for chunk in input.chunks(rate as usize / 10 * 2) {
    resampler.push(chunk, &mut output);
  }

  output
}

/// The root mean square of the left channel of `samples` from frame `from` on.
fn rms_left(samples: &[f32], from: usize) -> f64 {
  let lefts: Vec<f64> = samples
    .iter()
    .step_by(2)
    .skip(from)
    .map(|sample| f64::from(*sample))
    .collect();

  (lefts.iter().map(|sample| sample * sample).sum::<f64>() / lefts.len() as f64).sqrt()
}

#[test]
fn takes_rates_aac_takes_as_they_are() {
  assert!(RecordResampler::for_rate(48_000).is_none());
  assert!(RecordResampler::for_rate(44_100).is_none());
  assert_eq!(
    RecordResampler::for_rate(96_000).map(|resampler| resampler.get_input_rate()),
    Some(96_000)
  );
}

// A click lands on the output frame its time falls on, from a whole multiple of 48 kHz and from one that is not.
#[test]
fn keeps_a_click_where_its_time_falls() {
  for rate in [96_000, 88_200, 192_000, 32_000] {
    let mut input: Vec<f32> = vec![0.0; rate as usize * 2];

    // At 0.1 s.
    input[rate as usize / 10 * 2] = 1.0;

    let output: Vec<f32> = resample(rate, &input);
    let peak: usize = output
      .iter()
      .step_by(2)
      .enumerate()
      .max_by(|(_, left), (_, right)| left.total_cmp(right))
      .map(|(frame, _)| frame)
      .expect("a frame out");

    // What the taps still hold, 16 input frames, comes with the next input.
    let held: usize = (16 * 48_000_usize).div_ceil(rate as usize) + 1;

    assert_eq!(peak, 4_800, "from {rate} Hz");
    assert!(
      (output.len() / 2).abs_diff(48_000) <= held,
      "from {rate} Hz: {} frames",
      output.len() / 2
    );
  }
}

// A tone under 24 kHz keeps its level; one above it, which 48 kHz cannot hold, is filtered out rather than folded
// down.
#[test]
fn keeps_a_tone_and_filters_out_what_48_khz_cannot_hold() {
  let kept: f64 = rms_left(&resample(96_000, &sine(96_000, 96_000, 1_000.0)), 100);
  let folded: f64 = rms_left(&resample(96_000, &sine(96_000, 96_000, 30_000.0)), 100);
  let level: f64 = 0.5 / 2.0_f64.sqrt();

  assert!((kept / level - 1.0).abs() < 0.01, "a 1 kHz tone at {kept}, {level} in");
  assert!(
    20.0 * (folded / level).log10() < -40.0,
    "a 30 kHz tone at {folded}, {level} in"
  );
}
