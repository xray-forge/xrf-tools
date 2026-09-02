//! Chunks 14 and 15 - the skeleton animation pair, shared by both containers that carry it.
//!
//! Unlike every other top-level module here, this one owns no file type. `OGF_S_MOTIONS` and `OGF_S_SMPARAMS`
//! (`xray-16/src/xrCore/FMesh.hpp`) are a chunk family rather than a format: `motions_value::load` reads the pair
//! without caring whether an ogf or an omf supplied it (`xray-16/src/xrCore/Animation/SkeletonMotions.cpp`), and a
//! self-animated ogf is loaded through exactly that path. So [`crate::OgfFile`] and [`crate::OmfFile`] both reach
//! here, and neither one owns what they find.

pub(crate) mod chunks;
