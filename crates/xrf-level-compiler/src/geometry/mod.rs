//! A level's static geometry from `build.prj`, as xrLC shapes it before it splits it by material: loaded and checked,
//! welded, faces it refuses or drops sorted out, long edges cut, and vertices split by smoothing with their normals; and the
//! two collision forms built from it, the game's and the compilers' ray model.

pub(crate) mod geometry_base_coordinates;
pub(crate) mod geometry_collector;
pub(crate) mod geometry_collision;
pub(crate) mod geometry_face;
pub(crate) mod geometry_mesh;
pub(crate) mod geometry_mesh_load;
pub(crate) mod geometry_mesh_optimize;
pub(crate) mod geometry_mesh_tessellate;
pub(crate) mod geometry_normals;
pub(crate) mod geometry_optimization;
pub(crate) mod geometry_placement;
pub(crate) mod geometry_ray_model;
