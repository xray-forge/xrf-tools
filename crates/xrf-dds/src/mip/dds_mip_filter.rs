/// Resampling kernel a mip chain is reduced with.
///
/// The family the X-Ray converter offers (`kMIPFilter*`, `ETextureParams.h`), minus `Advanced`, which names no kernel
/// at all: it selects the SDK's own chain with its per-mip fade (`xrDXT/DXT.cpp`), so it belongs to the recipe that
/// drives this crate rather than to the resampler.
///
/// Bit-exact parity with the converter is not obtainable. Every one of these went to `nvDXTlibMTDLL.lib`, a closed
/// library, so each kernel here is its published mathematical definition rather than a port, cited where it is not
/// obvious, and pinned by arithmetic. Only `Box`, `Triangle` and `Kaiser` have any corpus witness to compare against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DdsMipFilter {
  Box,
  Point,
  Triangle,
  Quadratic,
  Cubic,
  Catrom,
  Mitchell,
  Gaussian,
  Sinc,
  Bessel,
  Hanning,
  Hamming,
  Blackman,
  Kaiser,
}

use crate::mip::bessel::{bessel_i0, bessel_j1};

/// Shape parameter of the Kaiser window, the value NVIDIA's own mip generator uses.
const KAISER_ALPHA: f64 = 4.0;

impl DdsMipFilter {
  /// Every kernel, in the order the SDK's own filter combo lists them.
  pub const NAMED: [Self; 14] = [
    Self::Point,
    Self::Box,
    Self::Triangle,
    Self::Quadratic,
    Self::Cubic,
    Self::Catrom,
    Self::Mitchell,
    Self::Gaussian,
    Self::Sinc,
    Self::Bessel,
    Self::Hanning,
    Self::Hamming,
    Self::Blackman,
    Self::Kaiser,
  ];

  /// Name of the kernel, matching the SDK's own token for it.
  pub const fn label(self) -> &'static str {
    match self {
      Self::Box => "Box",
      Self::Point => "Point",
      Self::Triangle => "Triangle",
      Self::Quadratic => "Quadratic",
      Self::Cubic => "Cubic",
      Self::Catrom => "Catrom",
      Self::Mitchell => "Mitchell",
      Self::Gaussian => "Gaussian",
      Self::Sinc => "Sinc",
      Self::Bessel => "Bessel",
      Self::Hanning => "Hanning",
      Self::Hamming => "Hamming",
      Self::Blackman => "Blackman",
      Self::Kaiser => "Kaiser",
    }
  }

  /// The kernel a caller named, matched against [`Self::label`] without regard to case.
  ///
  /// Here rather than beside each caller, because the CLI's `--mip-filter` and the editor's filter list are the same
  /// question asked twice: a name accepted by one and refused by the other would be a difference nothing states.
  pub fn from_label(label: &str) -> Option<Self> {
    Self::NAMED
      .into_iter()
      .find(|filter| filter.label().eq_ignore_ascii_case(label))
  }

  /// Half-width of the kernel's window, in destination pixels.
  ///
  /// Everything past it weighs nothing, so this is what decides how many source pixels each destination pixel reads.
  pub const fn support(self) -> f64 {
    match self {
      // One source pixel per destination pixel, so no window at all.
      Self::Point => 0.0,
      // Exactly the pixels that fall inside the destination pixel's footprint: at a halving, the 2x2 average.
      Self::Box => 0.5,
      Self::Triangle | Self::Hanning | Self::Hamming | Self::Blackman => 1.0,
      Self::Quadratic => 1.5,
      Self::Cubic | Self::Catrom | Self::Mitchell | Self::Gaussian => 2.0,
      Self::Kaiser => 3.0,
      // ImageMagick's window for the jinc filter, the first zero it rounds off at.
      Self::Bessel => 3.2383,
      Self::Sinc => 4.0,
    }
  }

  /// Weight the kernel gives a sample `distance` destination pixels from the centre.
  ///
  /// Weights are not normalized here: a resampler sums the ones inside its window and divides by that sum, which is
  /// what keeps a partial window at an edge from darkening the result.
  pub fn weight(self, distance: f64) -> f64 {
    let x: f64 = distance.abs();

    if self != Self::Point && x > self.support() {
      return 0.0;
    }

    match self {
      Self::Point => f64::from(u8::from(x < 0.5)),
      Self::Box => 1.0,
      Self::Triangle => 1.0 - x,
      // The quadratic B-spline, the degree-two member of the same family as `Cubic`.
      Self::Quadratic => {
        if x < 0.5 {
          0.75 - x * x
        } else {
          0.5 * (x - 1.5) * (x - 1.5)
        }
      }
      // Mitchell and Netravali, "Reconstruction Filters in Computer Graphics" (SIGGRAPH 1988), whose two parameters
      // place all three of these on one curve: the cubic B-spline, Catmull-Rom, and the paper's own recommendation.
      Self::Cubic => mitchell_netravali(x, 1.0, 0.0),
      Self::Catrom => mitchell_netravali(x, 0.0, 0.5),
      Self::Mitchell => mitchell_netravali(x, 1.0 / 3.0, 1.0 / 3.0),
      // `exp(-x^2 / 2 sigma^2)` at the half-pixel deviation resampling conventionally uses.
      Self::Gaussian => (-2.0 * x * x).exp(),
      Self::Sinc => sinc(x),
      // The jinc, sinc's radially symmetric counterpart, normalized to one at the centre.
      Self::Bessel => {
        if x == 0.0 {
          1.0
        } else {
          2.0 * bessel_j1(std::f64::consts::PI * x) / (std::f64::consts::PI * x)
        }
      }
      // Three raised-cosine windows, used here as kernels in their own right as the SDK offers them.
      Self::Hanning => 0.5 + 0.5 * (std::f64::consts::PI * x).cos(),
      Self::Hamming => 0.54 + 0.46 * (std::f64::consts::PI * x).cos(),
      Self::Blackman => 0.42 + 0.5 * (std::f64::consts::PI * x).cos() + 0.08 * (2.0 * std::f64::consts::PI * x).cos(),
      // A sinc closed by the Kaiser window, which is the shape a mip chain is conventionally reduced with.
      Self::Kaiser => {
        let ratio: f64 = x / self.support();

        sinc(x) * bessel_i0(KAISER_ALPHA * (1.0 - ratio * ratio).max(0.0).sqrt()) / bessel_i0(KAISER_ALPHA)
      }
    }
  }
}

/// The Mitchell-Netravali cubic at `x`, for the `b` and `c` that name one member of the family.
fn mitchell_netravali(x: f64, b: f64, c: f64) -> f64 {
  let x2: f64 = x * x;
  let x3: f64 = x2 * x;

  if x < 1.0 {
    ((12.0 - 9.0 * b - 6.0 * c) * x3 + (-18.0 + 12.0 * b + 6.0 * c) * x2 + (6.0 - 2.0 * b)) / 6.0
  } else {
    ((-b - 6.0 * c) * x3 + (6.0 * b + 30.0 * c) * x2 + (-12.0 * b - 48.0 * c) * x + (8.0 * b + 24.0 * c)) / 6.0
  }
}

/// The normalized sinc, `sin(pi x) / (pi x)`, continuous at the origin.
fn sinc(x: f64) -> f64 {
  if x == 0.0 {
    1.0
  } else {
    let scaled: f64 = std::f64::consts::PI * x;

    scaled.sin() / scaled
  }
}

#[cfg(test)]
mod tests {
  use super::DdsMipFilter;

  /// Weights are compared to the arithmetic in the comments, so the tolerance only has to absorb `f64` rounding.
  const EPSILON: f64 = 1e-9;

  fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
      (actual - expected).abs() < 1e-6,
      "{what}: expected {expected}, got {actual}"
    );
  }

  #[test]
  fn every_kernel_peaks_at_its_centre_and_is_symmetric() {
    // Two properties every member of the family has, so a kernel that failed either would be wrong whatever its
    // arithmetic says. Point is the one that is not a curve: it answers for a single sample, not a window.
    for filter in DdsMipFilter::NAMED {
      assert_close(filter.weight(0.0), filter.weight(-0.0), filter.label());

      for step in 1..12 {
        let distance: f64 = f64::from(step) * 0.25;

        assert_close(filter.weight(distance), filter.weight(-distance), filter.label());
        assert!(
          filter.weight(distance) <= filter.weight(0.0) + EPSILON,
          "{}: {distance} weighs more than the centre",
          filter.label()
        );
      }
    }
  }

  #[test]
  fn every_kernel_weighs_nothing_past_its_support() {
    for filter in DdsMipFilter::NAMED {
      let past: f64 = filter.support() + 0.5;

      assert_eq!(filter.weight(past), 0.0, "{}", filter.label());
      assert_eq!(filter.weight(-past), 0.0, "{}", filter.label());
    }
  }

  #[test]
  fn the_box_and_point_kernels_answer_one_inside_a_half_pixel() {
    // The two that are steps rather than curves: everything within half a pixel counts fully and nothing else counts
    // at all. At a halving that makes Box the 2x2 average the SDK's own chain builder uses.
    for filter in [DdsMipFilter::Box, DdsMipFilter::Point] {
      assert_eq!(filter.weight(0.0), 1.0, "{}", filter.label());
      assert_eq!(filter.weight(0.4), 1.0, "{}", filter.label());
      assert_eq!(filter.weight(0.6), 0.0, "{}", filter.label());
    }

    assert_eq!(DdsMipFilter::Box.support(), 0.5);
    assert_eq!(DdsMipFilter::Point.support(), 0.0);
  }

  #[test]
  fn the_triangle_kernel_falls_off_linearly() {
    assert_close(DdsMipFilter::Triangle.weight(0.0), 1.0, "triangle at 0");
    assert_close(DdsMipFilter::Triangle.weight(0.25), 0.75, "triangle at 0.25");
    assert_close(DdsMipFilter::Triangle.weight(0.5), 0.5, "triangle at 0.5");
    assert_close(DdsMipFilter::Triangle.weight(1.0), 0.0, "triangle at 1");
  }

  #[test]
  fn the_quadratic_kernel_meets_itself_where_its_branches_join() {
    // `0.75 - x^2` up to a half pixel and `0.5 (x - 1.5)^2` after it. Both give 0.5 at the join, which is the one
    // place a transcription error shows as a step.
    assert_close(DdsMipFilter::Quadratic.weight(0.0), 0.75, "quadratic at 0");
    assert_close(DdsMipFilter::Quadratic.weight(0.5), 0.5, "quadratic at 0.5");
    assert_close(DdsMipFilter::Quadratic.weight(1.0), 0.125, "quadratic at 1");
    assert_close(DdsMipFilter::Quadratic.weight(1.5), 0.0, "quadratic at 1.5");
  }

  #[test]
  fn the_three_cubics_sit_where_their_parameters_put_them() {
    // One curve, three points on it. Cubic is the B-spline, which does not interpolate: it weighs its own centre
    // 4/6 and its neighbour 1/6. Catrom does interpolate, so it weighs the centre fully and the neighbour nothing.
    // Mitchell is between them at 8/9.
    assert_close(DdsMipFilter::Cubic.weight(0.0), 4.0 / 6.0, "cubic at 0");
    assert_close(DdsMipFilter::Cubic.weight(1.0), 1.0 / 6.0, "cubic at 1");
    assert_close(DdsMipFilter::Cubic.weight(2.0), 0.0, "cubic at 2");

    assert_close(DdsMipFilter::Catrom.weight(0.0), 1.0, "catrom at 0");
    assert_close(DdsMipFilter::Catrom.weight(0.5), 0.5625, "catrom at 0.5");
    assert_close(DdsMipFilter::Catrom.weight(1.0), 0.0, "catrom at 1");

    assert_close(DdsMipFilter::Mitchell.weight(0.0), 8.0 / 9.0, "mitchell at 0");
    assert_close(DdsMipFilter::Mitchell.weight(2.0), 0.0, "mitchell at 2");
  }

  #[test]
  fn the_gaussian_and_sinc_kernels_match_their_definitions() {
    assert_close(DdsMipFilter::Gaussian.weight(0.0), 1.0, "gaussian at 0");
    assert_close(DdsMipFilter::Gaussian.weight(1.0), (-2.0_f64).exp(), "gaussian at 1");

    assert_close(DdsMipFilter::Sinc.weight(0.0), 1.0, "sinc at 0");
    assert_close(
      DdsMipFilter::Sinc.weight(0.5),
      2.0 / std::f64::consts::PI,
      "sinc at 0.5",
    );
    // `sin(pi)` is zero, so every whole pixel away from the centre weighs nothing.
    for step in 1..4 {
      assert_close(DdsMipFilter::Sinc.weight(f64::from(step)), 0.0, "sinc at a whole pixel");
    }
  }

  #[test]
  fn the_three_cosine_windows_match_their_coefficients() {
    // Each is `a0 + a1 cos(pi x) [+ a2 cos(2 pi x)]`, so a whole pixel away reduces to the alternating sum of them.
    assert_close(DdsMipFilter::Hanning.weight(0.0), 1.0, "hanning at 0");
    assert_close(DdsMipFilter::Hanning.weight(0.5), 0.5, "hanning at 0.5");
    assert_close(DdsMipFilter::Hanning.weight(1.0), 0.0, "hanning at 1");

    assert_close(DdsMipFilter::Hamming.weight(0.0), 1.0, "hamming at 0");
    assert_close(DdsMipFilter::Hamming.weight(1.0), 0.08, "hamming at 1");

    assert_close(DdsMipFilter::Blackman.weight(0.0), 1.0, "blackman at 0");
    assert_close(DdsMipFilter::Blackman.weight(1.0), 0.0, "blackman at 1");
  }

  #[test]
  fn the_bessel_and_kaiser_kernels_are_anchored_at_their_known_points() {
    assert_close(DdsMipFilter::Bessel.weight(0.0), 1.0, "bessel at 0");
    // The jinc's first zero is where `J1` first returns, at `pi x = 3.8317`.
    assert_close(
      DdsMipFilter::Bessel.weight(3.831_705_970 / std::f64::consts::PI),
      0.0,
      "bessel at its first zero",
    );

    assert_close(DdsMipFilter::Kaiser.weight(0.0), 1.0, "kaiser at 0");
    // The window closes at the support, and the sinc is zero at every whole pixel anyway.
    assert_close(DdsMipFilter::Kaiser.weight(3.0), 0.0, "kaiser at its support");
  }
}
