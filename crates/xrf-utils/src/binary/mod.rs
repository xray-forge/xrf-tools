//! What a binary format reader or writer checks before it trusts a count, a length or a field.

mod assertions;
mod capacity;
mod content_hash;
mod format_width;
mod positional_read;

pub use assertions::{assert, assert_equal, assert_length, assert_not_equal};
pub use capacity::{assert_count_fits, new_bounded_vec, new_declared_vec, to_format_size};
pub use content_hash::ContentHash;
pub use format_width::FormatWidth;
pub use positional_read::read_exact_at;
