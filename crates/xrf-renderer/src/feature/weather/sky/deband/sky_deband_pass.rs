use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, PipelineSet};

use crate::feature::weather::sky::deband::sky_deband_parameters::{SkyDebandCubeParameters, SkyDebandFlatParameters};
use crate::pass::build::pipeline_source::PipelineSource;

/// Smooths the sky's textures where their stored steps band into float copies (`frame/sky_deband_cube.wgsl`,
/// `frame/sky_deband_flat.wgsl`): a cube's six faces, or a cloud texture.
pub struct SkyDebandPass {
  cube_layout: wgpu::BindGroupLayout,
  flat_layout: wgpu::BindGroupLayout,
  cube: PipelineSet<wgpu::ComputePipeline>,
  flat: PipelineSet<wgpu::ComputePipeline>,
}

impl SkyDebandPass {
  /// Invocations a workgroup runs a side, as the shaders declare them.
  const WORKGROUP: u32 = 8;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let cube_layout: wgpu::BindGroupLayout = SkyDebandCubeParameters::create_layout(source.device);
    let flat_layout: wgpu::BindGroupLayout = SkyDebandFlatParameters::create_layout(source.device);

    Ok(Self {
      cube: PipelineSet::new(source.cache, Self::create_cube(source, &cube_layout)?),
      flat: PipelineSet::new(source.cache, Self::create_flat(source, &flat_layout)?),
      cube_layout,
      flat_layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping the last
  /// ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    let cube: XrfResult = self
      .cube
      .refresh(source.cache, || Self::create_cube(source, &self.cube_layout));
    let flat: XrfResult = self
      .flat
      .refresh(source.cache, || Self::create_flat(source, &self.flat_layout));

    if let Err(error) = cube.and(flat) {
      log::error!("Sky debanding rejected, smoothing with the last one: {error}");
    }
  }

  /// Smooths a cube into its copy, `size` texels a face side.
  pub fn record_cube(&self, context: &mut ComputeContext<'_>, size: u32, parameters: &SkyDebandCubeParameters<'_>) {
    context.bind(parameters);
    context.get_pass().set_pipeline(&self.cube);
    context
      .get_pass()
      .dispatch_workgroups(size.div_ceil(Self::WORKGROUP), size.div_ceil(Self::WORKGROUP), 6);
  }

  /// Smooths a cloud texture into its copy, `size` texels across and down.
  pub fn record_flat(
    &self,
    context: &mut ComputeContext<'_>,
    (width, height): (u32, u32),
    parameters: &SkyDebandFlatParameters<'_>,
  ) {
    context.bind(parameters);
    context.get_pass().set_pipeline(&self.flat);
    context
      .get_pass()
      .dispatch_workgroups(width.div_ceil(Self::WORKGROUP), height.div_ceil(Self::WORKGROUP), 1);
  }

  fn create_cube(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("sky deband cube", "frame/sky_deband_cube", "cs_deband_cube")
        .with_bind_group_layouts(&[Some(layout)]),
    )
  }

  fn create_flat(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("sky deband flat", "frame/sky_deband_flat", "cs_deband_flat")
        .with_bind_group_layouts(&[Some(layout)]),
    )
  }
}
