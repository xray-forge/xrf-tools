#![doc = include_str!("../README.md")]

mod compiler_textures;
mod detail;
mod level_compile_report;
mod level_compile_request;
mod level_compile_step;
mod level_compile_step_run;
mod level_compiler;
mod sdk;
mod shadow;

#[cfg(test)]
mod tests;

pub use crate::detail::detail_light_request::DetailLightRequest;
pub use crate::detail::detail_lighter::DetailLighter;
pub use crate::detail::detail_lighting::DetailLighting;
pub use crate::detail::detail_slot_counts::DetailSlotCounts;
pub use crate::level_compile_report::{LevelCompileReport, LevelCompileStepReport};
pub use crate::level_compile_request::LevelCompileRequest;
pub use crate::level_compile_step::LevelCompileStep;
pub use crate::level_compile_step_run::LevelCompileStepRun;
pub use crate::level_compiler::LevelCompiler;
pub use crate::shadow::shadow_class::ShadowClass;
pub use crate::shadow::shadow_class_counts::ShadowClassCounts;
pub use crate::shadow::shadow_materials::ShadowMaterials;
pub use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;
pub use crate::shadow::shadow_texture::ShadowTexture;
