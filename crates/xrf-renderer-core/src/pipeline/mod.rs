//! Shader permutations and the pipelines made from them: each described by a hashable description and made once.

mod compute_pipeline_description;
mod pipeline_cache;
mod pipeline_constants;
mod render_pipeline_description;
mod shader_override;
mod shader_permutation;
mod shader_source;
mod vertex_layout;

#[cfg(test)]
mod tests;

pub use compute_pipeline_description::ComputePipelineDescription;
pub use pipeline_cache::PipelineCache;
pub use pipeline_constants::PipelineConstants;
pub use render_pipeline_description::RenderPipelineDescription;
pub use shader_override::ShaderOverride;
pub use shader_permutation::ShaderPermutation;
pub use shader_source::ShaderSource;
pub use vertex_layout::VertexLayout;
