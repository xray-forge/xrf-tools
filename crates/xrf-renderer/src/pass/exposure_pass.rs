use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::frame::view_exposure::EXPOSURE_CELLS;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a measuring workgroup runs, as `shaders/frame/exposure.wgsl` declares them.
const MEASURE_WORKGROUP: u32 = 64;

/// `phase_luminance`: measures the scene combine finished, then adapts the scale the next frame's tonemap reads.
pub struct ExposurePass {
  layout: wgpu::BindGroupLayout,
  /// Measuring, then adapting.
  pipelines: [wgpu::ComputePipeline; 2],
  generation: u64,
}

impl ExposurePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = ExposureParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layout)?,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Exposure rejected, adapting with the last one: {error}"),
      }
    }
  }

  pub fn record(&self, context: &mut ComputeContext<'_>, parameters: &ExposureParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[0]);
    pass.dispatch_workgroups((EXPOSURE_CELLS * EXPOSURE_CELLS).div_ceil(MEASURE_WORKGROUP), 1, 1);
    pass.set_pipeline(&self.pipelines[1]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/exposure")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("exposure"),
      bind_group_layouts: &[Some(layout)],
      ..Default::default()
    });
    let create = |entry: &str| -> XrfResult<wgpu::ComputePipeline> {
      create_checked(device, entry, || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: Some(entry),
          layout: Some(&pipeline_layout),
          module: &module,
          entry_point: Some(entry),
          compilation_options: Default::default(),
          cache: None,
        })
      })
    };

    Ok([create("measure")?, create("adapt")?])
  }
}
