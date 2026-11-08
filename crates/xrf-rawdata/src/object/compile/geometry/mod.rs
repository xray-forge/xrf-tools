//! What the skeleton and the static export do alike: reading the editable mesh, packing a split's corners into
//! vertices, the tangent pass that splits and orders those, and cutting a split into children 16-bit indices reach.

pub(crate) mod export_vertex;
pub(crate) mod export_vertex_bone;
pub(crate) mod source;
pub(crate) mod split_partition;
pub(crate) mod tangent_basis;
pub(crate) mod vertex_packer;
pub(crate) mod vertex_packing;
