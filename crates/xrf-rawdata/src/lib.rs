#![doc = include_str!("../README.md")]

pub(crate) mod object;
pub(crate) mod provenance;

pub use crate::object::compile::object_compile_options::{ObjectCompileOptions, TangentMethod};
pub use crate::object::compile::object_compile_source::ObjectCompileSource;
pub use crate::object::compile::object_compiler::ObjectCompiler;
pub use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
pub use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;
pub use crate::object::compile::skeleton::skeleton_vertex::{SkeletonVertex, SkeletonVertexBone};
pub use crate::object::decompile::object_decompile::ObjectDecompile;
pub use crate::object::decompile::object_decompiler::ObjectDecompiler;
pub use crate::object::decompile::object_inference::ObjectInference;
pub use crate::object::verify::object_verification::{ObjectGeometryCensus, ObjectVerification};
pub use crate::object::verify::object_verifier::ObjectVerifier;
pub use crate::provenance::object_provenance::ObjectProvenance;
