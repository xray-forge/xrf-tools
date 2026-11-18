use std::f64::consts::PI;

/// Streams interleaved stereo from a device rate AAC does not take (it takes 44.1 and 48 kHz) to 48 kHz. A windowed
/// sinc low-passed under the lower rate's Nyquist, its taps centred on each output frame, makes output frame `n` the
/// input at `n × from / 48 kHz` with no time shift. Each call emits every output frame its taps reach and holds the
/// rest.
pub(crate) struct RecordResampler {
  from: u32,
  /// Input frames held for the taps; a stream starts with silence.
  held: Vec<[f32; 2]>,
  /// The next output frame's position among the held frames.
  position: f64,
  /// Input frames per output frame.
  step: f64,
  /// The low-pass cutoff as a share of the input's Nyquist.
  cutoff: f64,
}

impl RecordResampler {
  /// The output rate.
  pub(crate) const RATE: u32 = 48_000;

  /// Taps on each side of an output frame.
  const HALF_TAPS: usize = 16;

  /// The cutoff's share of the lower Nyquist, leaving the window room to roll off.
  const PASS: f64 = 0.9;

  /// A resampler from `rate`; none for a rate AAC takes as is, or zero.
  pub(crate) fn for_rate(rate: u32) -> Option<Self> {
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

  pub(crate) fn get_input_rate(&self) -> u32 {
    self.from
  }

  /// Drop held input so the next input starts a new stream, preceded by silence.
  pub(crate) fn restart(&mut self) {
    self.held.clear();
    self.held.resize(Self::HALF_TAPS, [0.0; 2]);
    self.position = Self::HALF_TAPS as f64;
  }

  /// Take interleaved stereo `input`, appending every 48 kHz frame its taps now reach to `output`.
  pub(crate) fn push(&mut self, input: &[f32], output: &mut Vec<f32>) {
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

    // Frames no later output frame's taps reach.
    let spent: usize = (self.position.floor() as usize + 1).saturating_sub(Self::HALF_TAPS);

    self.held.drain(..spent.min(self.held.len()));
    self.position -= spent as f64;
  }

  /// The weight of the tap `offset` input frames from an output frame: the low-pass sinc under a Blackman window.
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
