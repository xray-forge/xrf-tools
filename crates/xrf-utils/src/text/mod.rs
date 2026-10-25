//! Text as the workspace reads and writes it: encodings, line endings, natural order and small value lists.

mod encoding;
mod line_endings;
mod natural_order;
mod strings;

pub use encoding::{
  XRayEncoding, decode_bytes_from_base64, decode_bytes_to_string, decode_bytes_to_string_without_bom_handling,
  decode_string_from_base64, decode_w1251_bytes_to_string, encode_bytes_to_base64, encode_string_to_base64,
  encode_string_to_bytes, encode_string_to_w1251_bytes, new_utf8_encoder, new_windows1250_encoder,
  new_windows1251_encoder, new_windows1252_encoder, read_as_string_from_w1251_encoded,
};
pub use line_endings::LineEndings;
pub use natural_order::natural_cmp;
pub use strings::{
  error_to_string, read_engine_integer, to_camel_case, vector_from_string, vector_from_string_sized, vector_to_string,
};
