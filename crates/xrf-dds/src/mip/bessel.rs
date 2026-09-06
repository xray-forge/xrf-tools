//! Bessel functions the jinc and Kaiser kernels are defined in terms of.
//!
//! Their own module because they are mathematics rather than anything about a texture: published series with
//! published error bounds, pinned against published values, and the only numbers in the filter family that are an
//! approximation rather than a closed form.

/// Bessel function of the first kind, order one.
///
/// The polynomial approximations of Abramowitz and Stegun 9.4.4 and 9.4.6, whose stated error is below `1.3e-8` on the
/// near branch and `9e-8` on the far one - far tighter than the eight bits a weight is eventually applied to.
pub(crate) fn bessel_j1(x: f64) -> f64 {
  let magnitude: f64 = x.abs();

  let value: f64 = if magnitude < 3.0 {
    let t: f64 = (magnitude / 3.0) * (magnitude / 3.0);

    magnitude
      * (0.5
        + t
          * (-0.56249985
            + t * (0.21093573 + t * (-0.03954289 + t * (0.00443319 + t * (-0.00031761 + t * 0.00001109))))))
  } else {
    let t: f64 = 3.0 / magnitude;
    let amplitude: f64 = 0.79788456
      + t * (0.00000156 + t * (0.01659667 + t * (0.00017105 + t * (-0.00249511 + t * (0.00113653 + t * -0.00020033)))));
    let phase: f64 = magnitude - 2.35619449
      + t * (0.12499612 + t * (0.00005650 + t * (-0.00637879 + t * (0.00074348 + t * (0.00079824 + t * -0.00029166)))));

    amplitude * phase.cos() / magnitude.sqrt()
  };

  // Odd order, so the sign follows the argument.
  if x < 0.0 { -value } else { value }
}

/// Modified Bessel function of the first kind, order zero.
///
/// The polynomial approximations of Abramowitz and Stegun 9.8.1 and 9.8.2, stated to `1.6e-7` and `1.9e-7`.
pub(crate) fn bessel_i0(x: f64) -> f64 {
  let magnitude: f64 = x.abs();

  if magnitude < 3.75 {
    let t: f64 = (magnitude / 3.75) * (magnitude / 3.75);

    1.0 + t * (3.5156229 + t * (3.0899424 + t * (1.2067492 + t * (0.2659732 + t * (0.0360768 + t * 0.0045813)))))
  } else {
    let t: f64 = 3.75 / magnitude;

    magnitude.exp() / magnitude.sqrt()
      * (0.39894228
        + t
          * (0.01328592
            + t
              * (0.00225319
                + t
                  * (-0.00157565
                    + t * (0.00916281 + t * (-0.02057706 + t * (0.02635537 + t * (-0.01647633 + t * 0.00392377))))))))
  }
}

#[cfg(test)]
mod tests {
  use super::{bessel_i0, bessel_j1};

  fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
      (actual - expected).abs() < 1e-6,
      "{what}: expected {expected}, got {actual}"
    );
  }

  #[test]
  fn the_bessel_approximations_match_their_published_values() {
    // Both branches of each series, against values a table gives to ten digits. These are the only numbers in the
    // family that come from an approximation rather than from a closed form, so they are pinned directly.
    assert_close(bessel_j1(1.0), 0.440_050_585_7, "J1(1), near branch");
    assert_close(bessel_j1(2.0), 0.576_724_807_8, "J1(2), near branch");
    assert_close(bessel_j1(4.0), -0.066_043_328_0, "J1(4), far branch");
    assert_close(bessel_j1(-2.0), -0.576_724_807_8, "J1 is odd");

    assert_close(bessel_i0(0.0), 1.0, "I0(0)");
    assert_close(bessel_i0(1.0), 1.266_065_877_8, "I0(1), near branch");
    assert_close(bessel_i0(4.0), 11.301_921_953, "I0(4), far branch");
  }
}
