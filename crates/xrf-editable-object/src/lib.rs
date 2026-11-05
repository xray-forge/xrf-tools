#![doc = include_str!("../README.md")]

pub(crate) mod bone;
pub(crate) mod chunks;
pub(crate) mod editable_object_file;
pub(crate) mod mesh;
pub(crate) mod object;

#[cfg(test)]
mod tests;

pub use crate::bone::editable_bone::EditableBone;
pub use crate::bone::editable_bone_bind_pose::EditableBoneBindPose;
pub use crate::bone::editable_bone_break::EditableBoneBreak;
pub use crate::bone::editable_bone_definition::EditableBoneDefinition;
pub use crate::bone::editable_bone_joint::EditableBoneJoint;
pub use crate::bone::editable_bone_mass::EditableBoneMass;
pub use crate::chunks::editable_chunk_layout::EditableChunkLayout;
pub use crate::chunks::editable_extra_chunk::EditableExtraChunk;
pub use crate::editable_object_file::EditableObjectFile;
pub use crate::mesh::editable_face::EditableFace;
pub use crate::mesh::editable_face_corner::EditableFaceCorner;
pub use crate::mesh::editable_mesh::EditableMesh;
pub use crate::mesh::editable_mesh_bounds::EditableMeshBounds;
pub use crate::mesh::editable_surface_faces::EditableSurfaceFaces;
pub use crate::mesh::vmap::editable_vmap::EditableVMap;
pub use crate::mesh::vmap::editable_vmap_kind::EditableVMapKind;
pub use crate::mesh::vmap::editable_vmap_point::EditableVMapPoint;
pub use crate::mesh::vmap::editable_vmap_reference::EditableVMapReference;
pub use crate::object::editable_actor_transform::EditableActorTransform;
pub use crate::object::editable_bone_part::EditableBonePart;
pub use crate::object::editable_description::EditableDescription;
pub use crate::object::editable_surface::EditableSurface;
pub use crate::object::editable_time_width::EditableTimeWidth;
