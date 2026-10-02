use xrf_error::XrfResult;

use crate::frame::view_exposure::{EXPOSURE_CELLS, ViewExposure};
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::texture_binding;
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
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
    let compute: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("exposure"),
      entries: &[
        texture_entry(
          0,
          compute,
          wgpu::TextureSampleType::Float { filterable: false },
          wgpu::TextureViewDimension::D2,
        ),
        storage_entry(1, compute, true),
        uniform_entry(2, compute),
      ],
    });

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

  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    exposure: &ViewExposure,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("exposure"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.scene),
        wgpu::BindGroupEntry {
          binding: 1,
          resource: exposure.state.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: exposure.params.as_entire_binding(),
        },
      ],
    })
  }

  pub fn dispatch(&self, encoder: &mut wgpu::CommandEncoder, bind_group: &wgpu::BindGroup) {
    let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("exposure"),
      timestamp_writes: None,
    });

    pass.set_bind_group(0, bind_group, &[]);
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
