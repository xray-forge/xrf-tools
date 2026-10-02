use xrf_error::XrfResult;

use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a cull workgroup runs, as `shaders/static/cull.wgsl` declares them.
const WORKGROUP: u32 = 64;

/// The cull's entry points, in the order its pipelines are held.
const ENTRIES: [&str; 6] = [
  "cull_impostors",
  "cull_singles",
  "cull_rows",
  "clamp_counts",
  "cull_late",
  "clamp_late",
];

/// Decides each frame which clusters are drawn, into every batch's run of the visible list: an early phase before the
/// first draw, and a late one testing what the early phase set aside against the depth that draw left.
pub struct StaticCullPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipelines: [wgpu::ComputePipeline; 6],
  generation: u64,
}

impl StaticCullPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let compute: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("static cull"),
      entries: &[
        storage_entry(0, compute, false),
        storage_entry(1, compute, false),
        storage_entry(2, compute, false),
        storage_entry(3, compute, false),
        storage_entry(4, compute, false),
        storage_entry(5, compute, false),
        storage_entry(6, compute, true),
        storage_entry(7, compute, true),
        uniform_entry(8, compute),
        storage_entry(9, compute, true),
        storage_entry(10, compute, true),
        texture_entry(
          11,
          compute,
          wgpu::TextureSampleType::Float { filterable: false },
          wgpu::TextureViewDimension::D2,
        ),
        uniform_entry(12, compute),
        storage_entry(13, compute, false),
        storage_entry(14, compute, true),
        storage_entry(15, compute, true),
      ],
    });

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Static cull rejected, culling with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    scene: &StaticScene,
    params: &wgpu::Buffer,
    pyramid: &wgpu::TextureView,
    occlusion: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    let buffers: [(u32, &wgpu::Buffer); 15] = [
      (0, scene.clusters.get_buffer()),
      (1, scene.spheres.get_buffer()),
      (2, scene.slots.get_buffer()),
      (3, scene.places.get_buffer()),
      (4, scene.rows.get_buffer()),
      (5, &scene.regions),
      (6, scene.lists.get_buffer()),
      (7, &scene.args),
      (8, params),
      (9, scene.candidates.get_buffer()),
      (10, &scene.late),
      (12, occlusion),
      (13, scene.impostors.get_buffer()),
      (14, scene.terms.get_buffer()),
      (15, scene.impostor_list.get_buffer()),
    ];
    let mut entries: Vec<wgpu::BindGroupEntry<'_>> = buffers
      .iter()
      .map(|(binding, buffer)| wgpu::BindGroupEntry {
        binding: *binding,
        resource: buffer.as_entire_binding(),
      })
      .collect();

    entries.push(wgpu::BindGroupEntry {
      binding: 11,
      resource: wgpu::BindingResource::TextureView(pyramid),
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("static cull"),
      layout: &self.layout,
      entries: &entries,
    })
  }

  /// Decides each impostor's level of detail, culls every cluster and row by it, then clamps each batch's count to its
  /// run and sizes the late phase.
  pub fn dispatch_early(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
    params: &StaticCullParams,
  ) {
    let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("static cull"),
      timestamp_writes: None,
    });

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);

    for (pipeline, count) in [
      (&self.pipelines[0], params.impostor_count),
      (&self.pipelines[1], params.cluster_count),
      (&self.pipelines[2], params.row_count),
      (&self.pipelines[3], StaticBatch::COUNT as u32),
    ] {
      if count > 0 {
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(count.div_ceil(WORKGROUP), 1, 1);
      }
    }
  }

  /// Tests what the early phase set aside again, as many workgroups as it set aside, then clamps the late counts.
  pub fn dispatch_late(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
    scene: &StaticScene,
  ) {
    encoder.copy_buffer_to_buffer(
      &scene.late,
      StaticScene::LATE_DISPATCH_OFFSET,
      &scene.late_dispatch,
      0,
      12,
    );

    let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("static late cull"),
      timestamp_writes: None,
    });

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);
    pass.set_pipeline(&self.pipelines[4]);
    pass.dispatch_workgroups_indirect(&scene.late_dispatch, 0);
    pass.set_pipeline(&self.pipelines[5]);
    pass.dispatch_workgroups((StaticBatch::COUNT as u32).div_ceil(WORKGROUP), 1, 1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::ComputePipeline; 6]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/cull")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static cull"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
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

    let [impostors, singles, rows, clamp, late, clamp_late] = ENTRIES;

    Ok([
      create(impostors)?,
      create(singles)?,
      create(rows)?,
      create(clamp)?,
      create(late)?,
      create(clamp_late)?,
    ])
  }
}
