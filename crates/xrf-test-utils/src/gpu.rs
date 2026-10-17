use std::fmt::Display;

/// The variable a run sets where a GPU test must find a device, as Windows CI does with WARP.
pub const REQUIRE_GPU_VARIABLE: &str = "XRF_TEST_REQUIRE_GPU";

/// Skips a GPU test that found no device, saying why, or fails it where the run requires a device.
///
/// # Panics
///
/// Panics when `XRF_TEST_REQUIRE_GPU` is set to anything but `0`.
pub fn skip_without_gpu(reason: impl Display) {
  if std::env::var(REQUIRE_GPU_VARIABLE).is_ok_and(|value| value != "0") {
    panic!("No GPU, though {REQUIRE_GPU_VARIABLE} requires one: {reason}");
  }

  eprintln!("Skipped: {reason}");
}
