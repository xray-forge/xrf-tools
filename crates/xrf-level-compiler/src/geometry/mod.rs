//! A level's static geometry from `build.prj`, as xrLC shapes it before it splits it by material: loaded and checked,
//! welded, faces it refuses or drops sorted out, long edges cut, and vertices split by smoothing with their normals.

pub(crate) mod geometry_collector;
pub(crate) mod geometry_collision;
pub(crate) mod geometry_face;
pub(crate) mod geometry_mesh;
pub(crate) mod geometry_mesh_load;
pub(crate) mod geometry_mesh_optimize;
pub(crate) mod geometry_mesh_tessellate;
pub(crate) mod geometry_normals;
pub(crate) mod geometry_optimization;
