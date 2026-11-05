use std::f64::consts::PI;

/// Interleaved stereo sound brought from a device's rate to 48 kHz, for a device at a rate AAC does not take (it takes
/// 44.1 and 48 kHz): a windowed sinc low-passed under the lower rate's half, its taps centred on each output frame, so
/// output frame `n` is the input at `n × from / 48 kHz` with nothing shifted in time. It streams: each call takes the
/// next input and gives every output frame its taps reach, holding the rest for the next.
pub struct RecordResampler {
  from: u32,
  /// Input frames held for the taps, the stream's first preceded by silence.
  held: Vec<[f32; 2]>,
  /// Where the next output frame falls among the held frames.
  position: f64,
  /// Input frames an output frame steps over.
  step: f64,
  /// The low pass, a share of the input's half rate.
  cutoff: f64,
}

impl RecordResampler {
  /// The rate it brings sound to.
  pub const RATE: u32 = 48_000;

  /// Input frames each side of an output frame its taps reach.
  const HALF_TAPS: usize = 16;

  /// How far under the lower rate's half the low pass sits, leaving its window room to fall.
  const PASS: f64 = 0.9;

  /// A resampler from `rate`, none for a rate AAC takes as it is.
  pub fn for_rate(rate: u32) -> Option<Self> {
    if rate == 44_100 || rate == Self::RATE || rate == 0 {
      return None;
    }

    let step: f64 = f64::from(rate) / f64::from(Self::RATE);

    Some(Self {
      from: rate,
      held: vec![[0.0; 2]; Self::HALF_TAPS],
      position: Self::HALF_TAPS as f64,
      step,
      cutoff: Self::PASS * (1.0 / step).min(1.0),
    })
  }

  /// The rate it takes sound at.
  pub fn get_input_rate(&self) -> u32 {
    self.from
  }

  /// Forgets what it holds, its next input the start of a stream again, preceded by silence.
  pub fn restart(&mut self) {
    self.held.clear();
    self.held.resize(Self::HALF_TAPS, [0.0; 2]);
    self.position = Self::HALF_TAPS as f64;
  }

  /// Takes interleaved stereo `input`, appending to `output` every frame at 48 kHz its taps now reach.
  pub fn push(&mut self, input: &[f32], output: &mut Vec<f32>) {
    self.held.extend(input.as_chunks::<2>().0.iter().copied());

    while (self.position.floor() as usize) + Self::HALF_TAPS < self.held.len() {
      let centre: usize = self.position.floor() as usize;
      let fraction: f64 = self.position - centre as f64;
      let mut frame: [f64; 2] = [0.0; 2];

      for tap in centre + 1 - Self::HALF_TAPS..=centre + Self::HALF_TAPS {
        let weight: f64 = self.get_weight(tap as f64 - centre as f64 - fraction);
        let [left, right] = self.held[tap];

        frame[0] += f64::from(left) * weight;
        frame[1] += f64::from(right) * weight;
      }

      output.extend(frame.map(|sample| sample as f32));
      self.position += self.step;
    }

    // The frames no later output frame's taps reach.
    let spent: usize = (self.position.floor() as usize + 1).saturating_sub(Self::HALF_TAPS);

    self.held.drain(..spent.min(self.held.len()));
    self.position -= spent as f64;
  }

  /// The tap `offset` input frames from an output frame: the low pass's sinc under a Blackman window over the taps.
  fn get_weight(&self, offset: f64) -> f64 {
    let span: f64 = Self::HALF_TAPS as f64;

    if offset.abs() >= span {
      return 0.0;
    }

    let argument: f64 = PI * self.cutoff * offset;
    let sinc: f64 = if argument.abs() < 1e-12 {
      1.0
    } else {
      argument.sin() / argument
    };
    let phase: f64 = PI * (offset / span + 1.0);
    let window: f64 = 0.42 - 0.5 * phase.cos() + 0.08 * (2.0 * phase).cos();

    self.cutoff * sinc * window
  }
}
