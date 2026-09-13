//! The two file formats this crate reads and writes a texture as.
//!
//! Which of them a path names is what decides which writer runs, so the extensions belong beside the writers rather
//! than in a list of unrelated strings.

use xrf_extension::XrayExtension;

pub(crate) const DDS_EXTENSION: XrayExtension = XrayExtension::Dds;

pub(crate) const PNG_EXTENSION: XrayExtension = XrayExtension::Png;
