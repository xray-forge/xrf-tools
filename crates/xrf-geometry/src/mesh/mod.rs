//! A split's mesh as the SDK's exports and xrLC prepare it: corners packed into vertices, the tangent pass that splits
//! and orders them, cutting into children 16-bit indices reach, and ordering for the vertex cache.

pub(crate) mod mesh;
pub(crate) mod mesh_vertex;
pub(crate) mod mesh_vertex_bone;
pub(crate) mod split_partition;
pub(crate) mod tangent_basis;
pub(crate) mod vertex_cache_order;
pub(crate) mod vertex_packer;
pub(crate) mod vertex_packing;

#[cfg(test)]
mod tests;
