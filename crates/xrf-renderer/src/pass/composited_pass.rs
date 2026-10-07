use xrf_error::XrfResult;
use xrf_renderer_core::RasterContext;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::material_table::MaterialTable;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::shader::shader_library::ShaderLibrary;

/// Reversed depth steps a composited surface is pulled towards the eye by, so a surface laid on a wall never loses
/// to it, and the same scaled by its slope for one seen at a grazing angle.
/// The fragment entry compositing surfaces over the lit frame.
const COMPOSITED_FRAGMENT: &str = "fs_composited";

const DEPTH_BIAS: i32 = 256;
const SLOPE_BIAS: f32 = 1.0;

/// `out = colour + behind × second colour`, by dual-source blending: over by alpha, added and multiplied all in one.
const COMPOSITED_BLEND: wgpu::BlendState = wgpu::BlendState {
  color: wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::Src1,
    operation: wgpu::BlendOperation::Add,
  },
  alpha: wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::Src1Alpha,
    operation: wgpu::BlendOperation::Add,
  },
};

/// Draws a viewport's visible blended, added and multiplied static surfaces over its lit scene, and its wall marks into
/// its G-buffer's albedo before any light, each tested against the G-buffer's depth without writing it.
pub struct CompositedPass {
  layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 4],
  /// One a composited batch, in `StaticBatch::list_composited` order, then one a wall mark batch.
  pipelines: (Vec<wgpu::RenderPipeline>, Vec<wgpu::RenderPipeline>),
  generation: u64,
}

impl CompositedPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    scene_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    // A forward-drawn model is lit, and its sun read, per vertex.
    let lit: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let [table, sampler] = MaterialTable::get_layout_entries(2);
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("composited"),
      entries: &[
        uniform_entry(0, lit),
        storage_entry(1, fragment, false),
        table,
        sampler,
        texture_entry(
          4,
          lit,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2Array,
        ),
        uniform_entry(5, lit),
      ],
    });
    let layouts: [wgpu::BindGroupLayout; 4] = [
      view_layout.clone(),
      scene_layout.clone(),
      texture_layout.clone(),
      sky_layout.clone(),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts, &layout)?,
      layouts,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layouts, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Composited surfaces rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Binds what the surfaces read of their frame: the lighting, the exposure, the material table and the sun's shadow.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    table: &MaterialTable,
    (lighting, exposure): (&wgpu::Buffer, &wgpu::Buffer),
    shadows: &LevelShadows,
  ) -> wgpu::BindGroup {
    let [table, sampler] = table.get_entries(2);

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("composited"),
      layout: &self.layout,
      entries: &[
        buffer_binding(0, lighting),
        buffer_binding(1, exposure),
        table,
        sampler,
        texture_binding(4, &shadows.get_maps().view),
        buffer_binding(5, shadows.get_uniform()),
      ],
    })
  }

  /// Draws the composited surfaces each argument buffer lists, in cluster order.
  #[allow(clippy::too_many_arguments)]
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (composited_group, sky_group): (&wgpu::BindGroup, &wgpu::BindGroup),
    args: &[&wgpu::Buffer],
    (sorted, sorted_count): (Option<&StaticDrawParameters>, u32),
  ) {
    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);
    pass.set_bind_group(3, composited_group, &[]);
    pass.set_bind_group(4, sky_group, &[]);

    for (batch, pipeline) in StaticBatch::list_composited().zip(&self.pipelines.0) {
      // The models' are drawn back to front from the view's sorted list rather than as the cull listed them.
      if batch.layout == StaticLayout::Model {
        if let Some(sorted) = sorted
          && sorted_count > 0
        {
          context.bind(sorted);
          context.get_pass().set_pipeline(pipeline);
          context
            .get_pass()
            .draw(0..StaticScene::CLUSTER_VERTICES, 0..sorted_count);
        }

        continue;
      }

      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  /// Lays the wall marks each argument buffer lists into the G-buffer's albedo, before any light reads it.
  pub fn record_wallmarks(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &[&wgpu::Buffer],
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list_wallmarks().zip(&self.pipelines.1) {
      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [view_layout, scene_layout, texture_layout, sky_layout]: &[wgpu::BindGroupLayout; 4],
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(Vec<wgpu::RenderPipeline>, Vec<wgpu::RenderPipeline>)> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/composited")?;
    let composited_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("composited"),
      bind_group_layouts: &[
        Some(view_layout),
        Some(texture_layout),
        Some(scene_layout),
        Some(layout),
        Some(sky_layout),
      ],
      ..Default::default()
    });
    let wallmark_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("wall marks"),
      bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(scene_layout)],
      ..Default::default()
    });
    let composited = Self::create_batch_pipelines(
      device,
      &module,
      (&composited_layout, COMPOSITED_FRAGMENT),
      ViewTargets::SCENE,
      StaticBatch::list_composited(),
    )?;
    let wallmarks = Self::create_batch_pipelines(
      device,
      &module,
      (&wallmark_layout, "fs_wallmark"),
      ViewTargets::ALBEDO,
      StaticBatch::list_wallmarks(),
    )?;

    Ok((composited, wallmarks))
  }

  fn create_batch_pipelines(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    (pipeline_layout, fragment): (&wgpu::PipelineLayout, &str),
    format: wgpu::TextureFormat,
    batches: impl Iterator<Item = StaticBatch>,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
      format,
      blend: Some(COMPOSITED_BLEND),
      write_mask: wgpu::ColorWrites::ALL,
    })];

    batches
      .map(|batch| {
        // A model composited over the frame is lit by its vertices, as the engine's forward passes light it.
        let vertex: &str = match batch.layout {
          StaticLayout::Model if fragment == COMPOSITED_FRAGMENT => "vs_model_lit",
          layout => layout.get_vertex_entry(),
        };

        create_checked(device, fragment, || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(fragment),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
              module,
              entry_point: Some(vertex),
              compilation_options: Default::default(),
              buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
              module,
              entry_point: Some(fragment),
              compilation_options: Default::default(),
              targets: &targets,
            }),
            primitive: wgpu::PrimitiveState {
              front_face: wgpu::FrontFace::Ccw,
              cull_mode: Some(wgpu::Face::Back),
              ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
              format: ViewTargets::DEPTH,
              depth_write_enabled: Some(false),
              depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
              stencil: Default::default(),
              bias: wgpu::DepthBiasState {
                constant: DEPTH_BIAS,
                slope_scale: SLOPE_BIAS,
                clamp: 0.0,
              },
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
          })
        })
      })
      .collect()
  }
}
