//! The OGF visual format: what a model file carries, and the patches that rewrite one in place.

pub(crate) mod chunks;
pub(crate) mod data;
pub(crate) mod ogf_file;
pub mod ogf_motion_refs_processor;
pub(crate) mod ogf_raw_patch;
pub mod ogf_refs_patch_report;
pub mod ogf_texture_refs_processor;
pub mod residue;
pub mod survey;

#[cfg(test)]
mod tests;

pub use crate::chunks::ogf_bones_chunk::OgfBonesChunk;
pub use crate::chunks::ogf_children_chunk::OgfChildrenChunk;
pub use crate::chunks::ogf_children_link_chunk::OgfChildrenLinkChunk;
pub use crate::chunks::ogf_description_chunk::OgfDescriptionChunk;
pub use crate::chunks::ogf_geometry_container_chunk::OgfGeometryContainerChunk;
pub use crate::chunks::ogf_header_chunk::OgfHeaderChunk;
pub use crate::chunks::ogf_ik_data_chunk::OgfIkDataChunk;
pub use crate::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
pub use crate::chunks::ogf_lod_definition_chunk::OgfLodDefinitionChunk;
pub use crate::chunks::ogf_swi_data_chunk::OgfSwiDataChunk;
pub use crate::chunks::ogf_texture_chunk::OgfTextureChunk;
pub use crate::chunks::ogf_tree_definition_chunk::{OgfTreeColor, OgfTreeDefinitionChunk};
pub use crate::data::ogf_bone::OgfBone;
pub use crate::data::ogf_bone_ik_data::OgfBoneIkData;
pub use crate::data::ogf_bone_shape::OgfBoneShape;
pub use crate::data::ogf_box::*;
pub use crate::data::ogf_cylinder::OgfCylinder;
pub use crate::data::ogf_geometry::*;
pub use crate::data::ogf_joint_ik_data::OgfJointIkData;
pub use crate::data::ogf_joint_limit::OgfJointLimit;
pub use crate::data::ogf_lod_facet::OgfLodFacet;
pub use crate::data::ogf_lod_vertex::OgfLodVertex;
pub use crate::data::ogf_model_type::OgfModelType;
pub use crate::data::ogf_obb::OgfObb;
pub use crate::data::ogf_slide_window::*;
pub use crate::data::ogf_sphere::*;
pub use crate::data::ogf_vertex::*;
pub use crate::data::ogf_vertices::*;
pub use crate::data::rgb_color::RgbColor;
pub use crate::ogf_file::*;
pub use crate::ogf_motion_refs_processor::*;
pub use crate::ogf_refs_patch_report::*;
pub use crate::ogf_texture_refs_processor::*;
pub use crate::residue::*;
pub use crate::survey::*;
