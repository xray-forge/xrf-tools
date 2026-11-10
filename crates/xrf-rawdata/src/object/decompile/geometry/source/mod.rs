//! A compiled visual read as the decompile takes it: each surface's triangles, their corners' positions, normals and how
//! precisely those are stored, texture coordinates and bones.

pub(crate) mod bone_influences;
pub(crate) mod normal_precision;
pub(crate) mod ogf_corner;
pub(crate) mod ogf_surface;
pub(crate) mod surface_dress;
