//! A level's geometry from `build.prj`, as xrLC shapes it: loaded and checked, welded, faces it refuses or drops
//! sorted out, long edges cut, and vertices split by smoothing with their normals; the two collision forms built from
//! it, the game's and the compilers' ray model; and its visuals, static and multi-use.

pub(crate) mod geometry_base_coordinates;
pub(crate) mod geometry_basis;
pub(crate) mod geometry_bounds;
pub(crate) mod geometry_collector;
pub(crate) mod geometry_collision;
pub(crate) mod geometry_draft_light;
pub(crate) mod geometry_face;
pub(crate) mod geometry_hierarchy;
pub(crate) mod geometry_level;
pub(crate) mod geometry_level_census;
pub(crate) mod geometry_level_request;
pub(crate) mod geometry_lod;
pub(crate) mod geometry_lod_vertex;
pub(crate) mod geometry_mesh;
pub(crate) mod geometry_mesh_load;
pub(crate) mod geometry_mesh_model;
pub(crate) mod geometry_mesh_optimize;
pub(crate) mod geometry_mesh_tessellate;
pub(crate) mod geometry_model;
pub(crate) mod geometry_models;
pub(crate) mod geometry_node;
pub(crate) mod geometry_node_kind;
pub(crate) mod geometry_normals;
pub(crate) mod geometry_optimization;
pub(crate) mod geometry_packed_visual;
pub(crate) mod geometry_placement;
pub(crate) mod geometry_ray_model;
pub(crate) mod geometry_sector;
pub(crate) mod geometry_shader_table;
pub(crate) mod geometry_split_box;
pub(crate) mod geometry_splits;
pub(crate) mod geometry_step;
pub(crate) mod geometry_tangents;
pub(crate) mod geometry_tree;
pub(crate) mod geometry_visual;
pub(crate) mod geometry_visual_basis;
pub(crate) mod geometry_visual_vertex;
pub(crate) mod geometry_visuals;
pub(crate) mod geometry_visuals_request;
