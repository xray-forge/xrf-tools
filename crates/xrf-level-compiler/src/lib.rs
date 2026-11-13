#![doc = include_str!("../README.md")]

mod ai;
mod compiler_textures;
mod detail;
mod level_compile_report;
mod level_compile_request;
mod level_compile_step;
mod level_compile_step_run;
mod level_compiler;
mod ray;
mod sdk;
mod shadow;
mod spawn;

#[cfg(test)]
mod tests;

pub use crate::ai::ai_map::AiMap;
pub use crate::ai::ai_map_compilation::AiMapCompilation;
pub use crate::ai::ai_map_compiler::AiMapCompiler;
pub use crate::ai::ai_map_covers::AiMapCovers;
pub use crate::ai::ai_map_guid::AiMapGuid;
pub use crate::ai::ai_map_request::AiMapRequest;
pub use crate::ai::cover::ai_ray_model::AiRayModel;
pub use crate::detail::detail_light_request::DetailLightRequest;
pub use crate::detail::detail_lighter::DetailLighter;
pub use crate::detail::detail_lighting::DetailLighting;
pub use crate::detail::detail_slot_counts::DetailSlotCounts;
pub use crate::level_compile_report::{LevelCompileReport, LevelCompileStepReport};
pub use crate::level_compile_request::LevelCompileRequest;
pub use crate::level_compile_step::LevelCompileStep;
pub use crate::level_compile_step_run::LevelCompileStepRun;
pub use crate::level_compiler::LevelCompiler;
pub use crate::ray::ray_backend::RayBackend;
pub use crate::ray::ray_gpu::RayGpu;
pub use crate::shadow::shadow_class::ShadowClass;
pub use crate::shadow::shadow_class_counts::ShadowClassCounts;
pub use crate::shadow::shadow_materials::ShadowMaterials;
pub use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;
pub use crate::shadow::shadow_texture::ShadowTexture;
pub use crate::spawn::spawn_ai_map::SpawnAiMap;
pub use crate::spawn::spawn_cross_table::SpawnCrossTable;
pub use crate::spawn::spawn_game_edge::SpawnGameEdge;
pub use crate::spawn::spawn_game_graph_merge::SpawnGameGraphMerge;
pub use crate::spawn::spawn_game_graph_merger::SpawnGameGraphMerger;
pub use crate::spawn::spawn_graph_point::SpawnGraphPoint;
pub use crate::spawn::spawn_graph_point_removal::SpawnGraphPointRemoval;
pub use crate::spawn::spawn_graph_points::SpawnGraphPoints;
pub use crate::spawn::spawn_level_game_graph::SpawnLevelGameGraph;
pub use crate::spawn::spawn_merge_level::SpawnMergeLevel;
pub use crate::spawn::spawn_random::SpawnRandom;
pub use crate::spawn::spawn_removed_graph_point::SpawnRemovedGraphPoint;
pub use crate::spawn::spawn_skipped_connection::SpawnSkippedConnection;
