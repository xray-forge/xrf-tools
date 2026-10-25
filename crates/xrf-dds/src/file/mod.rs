//! What a DDS file holds: its header facts, its pixels, and the two directions between the two.

pub(crate) mod dds_file;
pub(crate) mod dds_format;
pub(crate) mod dds_metadata;
pub(crate) mod dds_png;
pub(crate) mod dds_uncompressed;

pub use dds_file::DdsFile;
pub use dds_format::DdsFormat;
pub use dds_metadata::DdsMetadata;
pub use dds_png::DdsPng;
