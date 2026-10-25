use std::fmt::Display;

/// Whether a GPU test may skip on a host without a device, or must fail there.
pub struct GpuRequirement;

impl GpuRequirement {
  /// The variable a run sets where a GPU test must find a device, as Windows CI does with WARP.
  pub const VARIABLE: &str = "XRF_TEST_REQUIRE_GPU";

  /// Skips a GPU test that found no device, saying why, or fails it where the run requires a device.
  ///
  /// # Panics
  ///
  /// Panics when `XRF_TEST_REQUIRE_GPU` is set to anything but `0`.
  pub fn skip_without_device(reason: impl Display) {
    if std::env::var(Self::VARIABLE).is_ok_and(|value| value != "0") {
      panic!("No GPU, though {} requires one: {reason}", Self::VARIABLE);
    }

    eprintln!("Skipped: {reason}");
  }
}
