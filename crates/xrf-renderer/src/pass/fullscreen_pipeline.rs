use xrf_error::XrfResult;
use xrf_renderer_core::{RenderPipelineDescription, ShaderPermutation};

use crate::pass::pipeline_source::PipelineSource;

/// A pass drawing one triangle over its whole target, its vertices from `common/fullscreen`, into one target: a format,
/// or a format with a blend.
pub fn create_fullscreen_pipeline(
  source: PipelineSource<'_>,
  module: &'static str,
  fragment: &'static str,
  layouts: &[Option<&wgpu::BindGroupLayout>],
  target: impl Into<wgpu::ColorTargetState>,
) -> XrfResult<wgpu::RenderPipeline> {
  create_fullscreen_pipeline_into(source, module, fragment, layouts, &[Some(target.into())])
}

/// The same, drawing into several targets at once.
pub fn create_fullscreen_pipeline_into(
  source: PipelineSource<'_>,
  module: &'static str,
  fragment: &'static str,
  layouts: &[Option<&wgpu::BindGroupLayout>],
  targets: &[Option<wgpu::ColorTargetState>],
) -> XrfResult<wgpu::RenderPipeline> {
  create_fullscreen_pipeline_with(source, module, fragment, &(), layouts, targets)
}

/// `create_fullscreen_pipeline_into` with the fragment's variant, its override constants, given by `permutation`.
pub fn create_fullscreen_pipeline_with<P: ShaderPermutation>(
  source: PipelineSource<'_>,
  module: &'static str,
  fragment: &'static str,
  permutation: &P,
  layouts: &[Option<&wgpu::BindGroupLayout>],
  targets: &[Option<wgpu::ColorTargetState>],
) -> XrfResult<wgpu::RenderPipeline> {
  source.get_render_pipeline(
    &RenderPipelineDescription::new(module, module, "vs_fullscreen")
      .with_fragment(fragment)
      .with_permutation(permutation)
      .with_bind_group_layouts(layouts)
      .with_targets(targets.to_vec()),
  )
}
