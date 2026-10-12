use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::composited_pass::COMPOSITED_DEPTH_BIAS;
use crate::pass::glass_parameters::GlassParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::shader::shader_library::ShaderLibrary;

/// Draws a viewport's visible glass as the enhanced glass draws it, over the frame and its copy taken just before:
/// what stands behind refracted, the sky reflected by the fresnel and the sun's highlight, written whole rather than
/// blended, tested against the G-buffer's depth as the composited surfaces are, without writing it.
pub struct GlassPass {
  layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 4],
  /// One a batch of either composited class, in `GlassPass::list_batches` order.
  pipelines: Vec<wgpu::RenderPipeline>,
  /// Reads the frame's copy between its texels, clamped at its edges.
  scene_sampler: wgpu::Sampler,
  generation: u64,
}

impl GlassPass {
  /// Where its pipelines hold the sky's group.
  const SKY_GROUP: u32 = 4;

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
    let layout: wgpu::BindGroupLayout = GlassParameters::create_layout(device);
    let layouts: [wgpu::BindGroupLayout; 4] = [
      view_layout.clone(),
      scene_layout.clone(),
      texture_layout.clone(),
      sky_layout.clone(),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts, &layout)?,
      layouts,
      scene_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("glass scene"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn get_scene_sampler(&self) -> &wgpu::Sampler {
    &self.scene_sampler
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layouts, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Glass rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Draws the glass among the composited surfaces each argument buffer lists, unsorted and strictly sorted, the
  /// models' too: each reads the frame's copy, so none needs what another drew.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (glass, sky): (&GlassParameters<'_>, &SkyParameters<'_>),
    args: &[&wgpu::Buffer],
  ) {
    context.bind(glass);
    context.bind_at(Self::SKY_GROUP, sky);
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in Self::list_batches().zip(&self.pipelines) {
      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  /// The batches glass may stand in: either composited class, of every layout.
  fn list_batches() -> impl Iterator<Item = StaticBatch> {
    StaticBatch::list().filter(|batch| batch.class.is_composited())
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [view_layout, scene_layout, texture_layout, sky_layout]: &[wgpu::BindGroupLayout; 4],
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/glass")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("glass"),
      bind_group_layouts: &[
        Some(view_layout),
        Some(texture_layout),
        Some(scene_layout),
        Some(layout),
        Some(sky_layout),
      ],
      ..Default::default()
    });
    let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(ViewTargets::SCENE.into())];

    Self::list_batches()
      .map(|batch| {
        create_checked(device, "fs_glass", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("glass"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
              module: &module,
              entry_point: Some(batch.layout.get_vertex_entry()),
              compilation_options: Default::default(),
              buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
              module: &module,
              entry_point: Some("fs_glass"),
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
              bias: COMPOSITED_DEPTH_BIAS,
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
