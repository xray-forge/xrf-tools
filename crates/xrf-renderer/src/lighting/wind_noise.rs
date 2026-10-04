use crate::weather::weather_random::WeatherRandom;

/// `PERLIN_SAMPLE_SIZE`: gradients in the table, and the mask that wraps into it.
const SAMPLES: usize = 256;

/// `N`: what the argument is lifted by before it is cut into the table's cells.
const LIFT: f32 = 4096.0;

/// `SetOctaves(2)` and `SetAmplitude(0.66666f)`, as `CEnvironment` sets its noise up.
const OCTAVES: usize = 2;
const AMPLITUDE: f32 = 0.66666;

/// `CPerlinNoise1D` as `CEnvironment` drives `wind_strength_factor` by it: two octaves of one-dimensional gradient noise
/// read continuously, each octave's time advanced by the step since the last read at the frequency of that moment, so a
/// change of frequency changes the pace without a jump. Its tables are filled from a seed of its own, not the C library's
/// `rand`, so its values are the engine's in kind rather than in number.
pub struct WindNoise {
  /// `p`, the cells' shuffled order, and `g1`, their gradients, each repeated past the end as the engine repeats them.
  order: [usize; SAMPLES * 2 + 2],
  gradients: [f32; SAMPLES * 2 + 2],
  /// `mTimes` and `mPrevContiniousTime`.
  times: [f32; OCTAVES],
  previous: f32,
}

impl WindNoise {
  /// `init`: the gradients in `[-1, 1)` and their order shuffled.
  pub fn new(seed: u64) -> Self {
    let mut random: WeatherRandom = WeatherRandom::new(seed);
    let mut order: [usize; SAMPLES * 2 + 2] = [0; SAMPLES * 2 + 2];
    let mut gradients: [f32; SAMPLES * 2 + 2] = [0.0; SAMPLES * 2 + 2];

    for index in 0..SAMPLES {
      order[index] = index;
      gradients[index] = random.between(-1.0, 1.0);
    }

    for index in (1..SAMPLES).rev() {
      let other: usize = ((random.next() * SAMPLES as f32) as usize).min(SAMPLES - 1);

      order.swap(index, other);
    }

    for index in 0..SAMPLES + 2 {
      order[SAMPLES + index] = order[index];
      gradients[SAMPLES + index] = gradients[index];
    }

    Self {
      order,
      gradients,
      times: [0.0; OCTAVES],
      previous: 0.0,
    }
  }

  /// `GetContinious`: the noise at a moment, in seconds, read at a frequency; within two thirds of zero either way.
  pub fn read(&mut self, time: f32, frequency: f32) -> f32 {
    let mut step: f32 = if self.previous != 0.0 {
      time - self.previous
    } else {
      time
    };
    let mut amplitude: f32 = AMPLITUDE;
    let mut result: f32 = 0.0;

    self.previous = time;
    step *= frequency;

    for octave in &mut self.times {
      *octave += step;
      result += Self::noise(&self.order, &self.gradients, *octave) * amplitude;
      step *= 2.0;
      amplitude *= 0.5;
    }

    result
  }

  /// `noise`: the gradients of the two cells around an argument, eased between.
  fn noise(order: &[usize], gradients: &[f32], argument: f32) -> f32 {
    let lifted: f32 = argument + LIFT;
    let first: usize = (lifted as i64 as usize) & (SAMPLES - 1);
    let second: usize = (first + 1) & (SAMPLES - 1);
    let into: f32 = lifted - lifted.trunc();
    let eased: f32 = into * into * (3.0 - 2.0 * into);
    let u: f32 = into * gradients[order[first]];
    let v: f32 = (into - 1.0) * gradients[order[second]];

    u + eased * (v - u)
  }
}
