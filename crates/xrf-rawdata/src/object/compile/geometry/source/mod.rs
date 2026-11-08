//! An editable mesh read as both exports read it: its face lists in the order they walk them, and each face corner's
//! point, normal and vertex map entries.

pub(crate) mod mesh_corner;
pub(crate) mod mesh_corners;
pub(crate) mod mesh_surface_faces;
pub(crate) mod vertex_map_entry;
