use xrf_error::XrfResult;

use crate::frame::depth_pyramid::DepthPyramid;
use crate::pass::layout_entries::texture_entry;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a reduction workgroup runs a side, as `shaders/frame/pyramid.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Reduces a viewport's depth into its pyramid, a dispatch a level.
pub struct DepthPyramidPass {
  depth_layout: wgpu::BindGroupLayout,
  level_layout: wgpu::BindGroupLayout,
  /// Reducing the depth into the first level, then a level into the next.
  pipelines: [wgpu::ComputePipeline; 2],
  generation: u64,
}

impl DepthPyramidPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let compute: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let target: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
      binding: 2,
      visibility: compute,
      ty: wgpu::BindingType::StorageTexture {
        access: wgpu::StorageTextureAccess::WriteOnly,
        format: DepthPyramid::FORMAT,
        view_dimension: flat,
      },
      count: None,
    };
    let depth_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("depth pyramid first level"),
      entries: &[texture_entry(0, compute, wgpu::TextureSampleType::Depth, flat), target],
    });
    let level_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("depth pyramid level"),
      entries: &[
        texture_entry(1, compute, wgpu::TextureSampleType::Float { filterable: false }, flat),
        target,
      ],
    });

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &depth_layout, &level_layout)?,
      generation: shaders.get_generation(),
      depth_layout,
      level_layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.depth_layout, &self.level_layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Depth pyramid rejected, reducing with the last one: {error}"),
      }
    }
  }

  /// A bind group a level: the first reading the depth, each next the level before it.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    depth: &wgpu::TextureView,
    pyramid: &DepthPyramid,
  ) -> Vec<wgpu::BindGroup> {
    (0..pyramid.levels as usize)
      .map(|level| {
        let (layout, source): (&wgpu::BindGroupLayout, wgpu::BindGroupEntry<'_>) = if level == 0 {
          (
            &self.depth_layout,
            wgpu::BindGroupEntry {
              binding: 0,
              resource: wgpu::BindingResource::TextureView(depth),
            },
          )
        } else {
          (
            &self.level_layout,
            wgpu::BindGroupEntry {
              binding: 1,
              resource: wgpu::BindingResource::TextureView(&pyramid.level_views[level - 1]),
            },
          )
        };

        device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: Some("depth pyramid"),
          layout,
          entries: &[
            source,
            wgpu::BindGroupEntry {
              binding: 2,
              resource: wgpu::BindingResource::TextureView(&pyramid.level_views[level]),
            },
          ],
        })
      })
      .collect()
  }

  pub fn record(&self, pass: &mut wgpu::ComputePass<'_>, pyramid: &DepthPyramid, bind_groups: &[wgpu::BindGroup]) {
    for (level, bind_group) in bind_groups.iter().enumerate() {
      let (width, height): (u32, u32) = pyramid.get_level_size(level as u32);

      pass.set_pipeline(&self.pipelines[(level > 0) as usize]);
      pass.set_bind_group(0, bind_group, &[]);
      pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    depth_layout: &wgpu::BindGroupLayout,
    level_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/pyramid")?;
    let create = |entry: &str, layout: &wgpu::BindGroupLayout| -> XrfResult<wgpu::ComputePipeline> {
      let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(entry),
        bind_group_layouts: &[Some(layout)],
        ..Default::default()
      });

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

    Ok([
      create("reduce_depth", depth_layout)?,
      create("reduce_level", level_layout)?,
    ])
  }
}
