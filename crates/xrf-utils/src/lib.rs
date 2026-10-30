#![doc = include_str!("../README.md")]

mod binary;
mod host_path;
mod staging;
mod text;
mod units;

pub use crate::binary::{
  ContentHash, FormatWidth, assert, assert_count_fits, assert_equal, assert_length, assert_not_equal, new_bounded_vec,
  new_declared_vec, read_exact_at, to_format_size,
};
pub use crate::host_path::{
  HostDestination, HostFileName, HostLinkPolicy, HostRelativePath, format_path, format_path_or, to_absolute_path,
  to_portable_path_string,
};
pub use crate::staging::StagedFile;
#[cfg(feature = "staging-faults")]
pub use crate::staging::StagingFaults;
pub use crate::text::{
  LineEndings, XRayEncoding, decode_bytes_from_base64, decode_bytes_to_string,
  decode_bytes_to_string_without_bom_handling, decode_string_from_base64, decode_w1251_bytes_to_string,
  encode_bytes_to_base64, encode_string_to_base64, encode_string_to_bytes, encode_string_to_w1251_bytes,
  error_to_string, natural_cmp, new_utf8_encoder, new_windows1250_encoder, new_windows1251_encoder,
  new_windows1252_encoder, read_as_string_from_w1251_encoded, read_engine_integer, to_camel_case, vector_from_string,
  vector_from_string_sized, vector_to_string,
};
pub use crate::units::{
  BYTES_PER_GIGABYTE, BYTES_PER_KILOBYTE, BYTES_PER_MEGABYTE, duration_ms, duration_ms_vec, duration_to_millis,
  format_bytes, format_bytes_pair, format_duration, megabytes_to_bytes, optional_duration_ms, wall_clock_millis,
};
