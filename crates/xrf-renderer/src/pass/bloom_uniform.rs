use crate::frame::view_targets::ViewTargets;

/// Taps each way of `bloom_filter.ps`, beside its centre.
const TAPS: usize = 7;

/// What `shaders/frame/bloom.wgsl` reads as its `Bloom`: one for the build and one for each way the bloom is blurred.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BloomUniform {
  /// The build: half a texel of the frame, then the threshold. A filter: one texel of the target along its way, then
  /// one where only the side behind is read.
  pub params: [f32; 4],
  /// `CalcGauss_wave`'s weights: the taps one to four out, then five to seven out and the centre.
  pub weights: [[f32; 4]; 2],
}

impl BloomUniform {
  /// `bloom_build.ps` over a frame drawn `size` texels, losing `threshold` of the summed brightness.
  pub fn build(size: (u32, u32), threshold: f32) -> Self {
    Self {
      params: [0.5 / size.0 as f32, 0.5 / size.1 as f32, threshold, 0.0],
      weights: [[0.0; 4]; 2],
    }
  }

  /// `bloom_filter.ps` across the target, its radius `kernel_g` texels: the same radius down, by the frame's height
  /// over its width, as the engine blurs it.
  pub fn filter(is_across: bool, (radius, strength): (f32, f32), aspect: f32, is_one_sided: bool) -> Self {
    let (width, height) = ViewTargets::BLOOM_SIZE;
    let (along, radius): ([f32; 2], f32) = if is_across {
      ([1.0 / width as f32, 0.0], radius)
    } else {
      ([0.0, 1.0 / height as f32], radius * aspect)
    };

    Self {
      params: [along[0], along[1], f32::from(u8::from(is_one_sided)), 0.0],
      weights: Self::get_wave(radius, strength),
    }
  }

  /// `CalcGauss_wave`: two kernels added, one of the radius and one of a third of it, each of the strength.
  pub fn get_wave(radius: f32, strength: f32) -> [[f32; 4]; 2] {
    let [base, detail] = [radius, radius / 3.0].map(|it| Self::get_kernel(it, strength));
    let weights: Vec<f32> = (0..=TAPS).map(|tap| base[tap] + detail[tap]).collect();

    [
      [weights[1], weights[2], weights[3], weights[4]],
      [weights[5], weights[6], weights[7], weights[0]],
    ]
  }

  /// `CalcGauss_k7`: the centre and seven taps out of a Gaussian of a radius, all fifteen summing to the strength.
  fn get_kernel(radius: f32, strength: f32) -> [f32; TAPS + 1] {
    // The console holds the radius at one or more; nothing below a hundredth reaches the division.
    let radius: f32 = radius.max(0.01);
    let weights: [f32; TAPS + 1] = std::array::from_fn(|tap| (-((tap * tap) as f32) / (2.0 * radius * radius)).exp());
    let total: f32 = weights[0] + 2.0 * weights[1..].iter().sum::<f32>();

    weights.map(|weight| strength * weight / total)
  }
}
