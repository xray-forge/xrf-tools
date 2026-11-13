#![doc = include_str!("../README.md")]

pub(crate) mod mesh;
pub(crate) mod ray;
pub(crate) mod sky;
pub(crate) mod tracer;
pub(crate) mod triangle;
pub(crate) mod weld;

pub use crate::mesh::mesh::Mesh;
pub use crate::mesh::mesh_vertex::MeshVertex;
pub use crate::mesh::mesh_vertex_bone::MeshVertexBone;
pub use crate::mesh::split_partition::SplitPartition;
pub use crate::mesh::tangent_basis::TangentBasis;
pub use crate::mesh::vertex_cache_order::VertexCacheOrder;
pub use crate::mesh::vertex_packer::VertexPacker;
pub use crate::mesh::vertex_packing::VertexPacking;
pub use crate::ray::facing::Facing;
pub use crate::ray::ray::Ray;
pub use crate::ray::ray_crossing::RayCrossing;
pub use crate::sky::hemisphere::Hemisphere;
pub use crate::sky::hemisphere_light::HemisphereLight;
pub use crate::tracer::bvh_tracer::BvhTracer;
pub use crate::tracer::trace_hit::TraceHit;
pub use crate::tracer::tracer::Tracer;
pub use crate::triangle::face_area::FaceArea;
pub use crate::triangle::triangle::Triangle;
pub use crate::weld::point_welder::PointWelder;
