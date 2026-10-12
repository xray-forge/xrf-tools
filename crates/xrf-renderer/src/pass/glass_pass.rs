use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::composited_pass::COMPOSITED_DEPTH_BIAS;
use crate::pass::glass_parameters::GlassParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;

/// Draws a viewport's visible glass as the enhanced glass draws it, over the frame and its copy taken just before:
/// what stands behind refracted, the sky reflected by the fresnel and the sun's highlight, written whole rather than
/// blended, tested against the G-buffer's depth as the composited surfaces are, without writing it.
pub struct GlassPass {
  layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 4],
  /// One a layout, in `StaticLayout::ALL` order.
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
    source: PipelineSource<'_>,
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
      pipelines: Self::create_pipelines(device, source, &layouts, &layout)?,
      layouts,
      scene_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("glass scene"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn get_scene_sampler(&self) -> &wgpu::Sampler {
    &self.scene_sampler
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(device, source, &self.layouts, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Glass rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Draws the glass each argument buffer lists, every layout's run of the glass draw list, unsorted and strictly
  /// sorted surfaces alike: each reads the frame's copy, so none needs what another drew.
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

    for (layout, pipeline) in StaticLayout::ALL.into_iter().zip(&self.pipelines) {
      let list: u64 = StaticBatch::get_pass_list(layout, StaticSurfacePass::Glass).into();

      context.bind(&layouts[layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, list * 16);
      }
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    [view_layout, scene_layout, texture_layout, sky_layout]: &[wgpu::BindGroupLayout; 4],
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let module: wgpu::ShaderModule = source.get_module("static/glass")?;
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

    StaticLayout::ALL
      .into_iter()
      .map(|layout| {
        create_checked(device, "fs_glass", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("glass"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
              module: &module,
              entry_point: Some(layout.get_vertex_entry()),
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
            cache: source.get_driver_cache(),
          })
        })
      })
      .collect()
  }
}
