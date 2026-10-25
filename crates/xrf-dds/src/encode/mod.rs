//! Writing a texture: the layout to write it in, and what that choice costs.

pub(crate) mod dds_encode_attempt;
pub(crate) mod dds_encode_candidate;
pub(crate) mod dds_encoding;
pub(crate) mod dds_image_difference;

pub use dds_encode_attempt::DdsEncodeAttempt;
pub use dds_encode_candidate::DdsEncodeCandidate;
pub use dds_encoding::DdsEncoding;
pub use dds_image_difference::DdsImageDifference;
