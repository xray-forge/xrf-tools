use std::f64::consts::TAU;

use crate::viewer::recording::writer::RecordResampler;

fn sine(rate: u32, frames: usize, frequency: f64) -> Vec<f32> {
  (0..frames)
    .flat_map(|frame| {
      let sample: f32 = ((TAU * frequency * frame as f64 / f64::from(rate)).sin() * 0.5) as f32;

      [sample, sample]
    })
    .collect()
}

// Fed a tenth of a second at a time, as a tap's blocks come.
fn resample(rate: u32, input: &[f32]) -> Vec<f32> {
  let mut resampler: RecordResampler = RecordResampler::for_rate(rate).expect("a rate AAC does not take");
  let mut output: Vec<f32> = Vec::new();

  for chunk in input.chunks(rate as usize / 10 * 2) {
    resampler.push(chunk, &mut output);
  }

  output
}

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

// Rates both whole multiples of 48 kHz and not.
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

    // The taps still hold 16 input frames, released by the next input.
    let held: usize = (16 * 48_000_usize).div_ceil(rate as usize) + 1;

    assert_eq!(peak, 4_800, "from {rate} Hz");
    assert!(
      (output.len() / 2).abs_diff(48_000) <= held,
      "from {rate} Hz: {} frames",
      output.len() / 2
    );
  }
}

// A 30 kHz tone must be filtered out, not aliased down.
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
