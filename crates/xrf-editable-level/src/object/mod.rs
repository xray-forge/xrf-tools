//! Typed scene objects, one per object tool, each reading and writing its sections as the editor's `SaveLTX` does.

pub(crate) mod editable_custom_object;
pub(crate) mod editable_glow_object;
pub(crate) mod editable_object_transform;
pub(crate) mod editable_particle_object;
pub(crate) mod editable_portal_object;
pub(crate) mod editable_scene_object;
pub(crate) mod editable_tool_object;
pub(crate) mod fog_volume;
pub(crate) mod light;
pub(crate) mod sector;
pub(crate) mod shape;
pub(crate) mod sound;
pub(crate) mod spawn;
pub(crate) mod way;
