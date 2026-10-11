use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::frame::froxel_volume::FroxelVolume;
use crate::pass::froxel_inject_parameters::FroxelInjectParameters;
use crate::pass::froxel_integrate_parameters::FroxelIntegrateParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Invocations a workgroup runs across and down the grid, as both source declare them.
const WORKGROUP: u32 = 8;

/// Draws the enhanced volumetric lighting's froxels: fills each with what the sun and the local lights scatter in it
/// and how thick it is, over its history, then integrates them along the view into what reaches the camera up to each,
/// which the shafts' composite reads at each pixel's depth.
pub struct FroxelPass {
  layouts: [wgpu::BindGroupLayout; 3],
  /// The fill, then the integration.
  pipelines: [wgpu::ComputePipeline; 2],
  /// Reads the history between its froxels, clamped at its edges.
  sampler: wgpu::Sampler,
  /// A volume of one froxel, scattering nothing, for the composite where none is drawn.
  empty: wgpu::TextureView,
  generation: u64,
}

impl FroxelPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: [wgpu::BindGroupLayout; 3] = [
      view_layout.clone(),
      FroxelInjectParameters::create_layout(device),
      FroxelIntegrateParameters::create_layout(device),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layouts)?,
      layouts,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("froxels"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      empty: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some("froxels none"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D3,
          format: FroxelVolume::FORMAT,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default()),
      generation: source.get_generation(),
    })
  }

  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Froxels rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Fills every froxel of the grid.
  pub fn record_inject(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    parameters: &FroxelInjectParameters,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();
    let [width, height, depth] = FroxelVolume::SIZE;

    pass.set_pipeline(&self.pipelines[0]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), depth);
  }

  /// Integrates every column of the grid along the view.
  pub fn record_integrate(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    parameters: &FroxelIntegrateParameters,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();
    let [width, height, _] = FroxelVolume::SIZE;

    pass.set_pipeline(&self.pipelines[1]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    [view_layout, inject_layout, integrate_layout]: &[wgpu::BindGroupLayout; 3],
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let create =
      |module: &'static str, entry: &'static str, layout: &wgpu::BindGroupLayout| -> XrfResult<wgpu::ComputePipeline> {
        source.get_compute_pipeline(
          &ComputePipelineDescription::new(entry, module, entry)
            .with_bind_group_layouts(&[Some(view_layout), Some(layout)]),
        )
      };

    Ok([
      create("frame/froxel_inject", "froxel_inject", inject_layout)?,
      create("frame/froxel_integrate", "froxel_integrate", integrate_layout)?,
    ])
  }
}
