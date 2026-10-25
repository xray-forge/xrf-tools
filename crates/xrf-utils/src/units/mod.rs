//! The workspace's units: durations as milliseconds on the wire, sizes as bytes, and the wall clock.

mod clock;
mod duration;
mod size;

pub use clock::wall_clock_millis;
pub use duration::{duration_ms, duration_ms_vec, duration_to_millis, format_duration, optional_duration_ms};
pub use size::{
  BYTES_PER_GIGABYTE, BYTES_PER_KILOBYTE, BYTES_PER_MEGABYTE, format_bytes, format_bytes_pair, megabytes_to_bytes,
};
