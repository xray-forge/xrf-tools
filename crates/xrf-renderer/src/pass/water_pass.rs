use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::shader::shader_library::ShaderLibrary;

/// What the distortion target holds where nothing distorts it, `(127, 127, 0, 127)` as the engine clears it.
const NEUTRAL_DISTORTION: f64 = 127.0 / 255.0;

/// Draws a viewport's visible water over its lit scene, tested against the G-buffer's depth without writing it, and the
/// distortion each surface causes into the distortion target, cleared to nothing first.
pub struct WaterPass {
  layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 3],
  /// One a water batch, in `StaticBatch::list_water` order.
  pipelines: Vec<wgpu::RenderPipeline>,
  generation: u64,
}

impl WaterPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    scene_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let cube: wgpu::TextureViewDimension = wgpu::TextureViewDimension::Cube;
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("water"),
      entries: &[
        uniform_entry(0, fragment),
        uniform_entry(1, fragment),
        texture_entry(
          2,
          fragment,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2,
        ),
        texture_entry(3, fragment, filtered, cube),
        texture_entry(4, fragment, filtered, cube),
        wgpu::BindGroupLayoutEntry {
          binding: 5,
          visibility: fragment,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let layouts: [wgpu::BindGroupLayout; 3] = [view_layout.clone(), scene_layout.clone(), texture_layout.clone()];

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
        Err(error) => log::error!("Water rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Binds what the water reads of its frame: the lighting, its own uniform, the G-buffer's depth and both skies.
  #[allow(clippy::too_many_arguments)]
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    lighting: &wgpu::Buffer,
    water: &wgpu::Buffer,
    skies: [&wgpu::TextureView; 2],
    sampler: &wgpu::Sampler,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("water"),
      layout: &self.layout,
      entries: &[
        buffer_binding(0, lighting),
        buffer_binding(1, water),
        texture_binding(2, &targets.depth),
        texture_binding(3, skies[0]),
        texture_binding(4, skies[1]),
        wgpu::BindGroupEntry {
          binding: 5,
          resource: wgpu::BindingResource::Sampler(sampler),
        },
      ],
    })
  }

  /// Clears the distortion target, then draws the water each argument buffer lists.
  #[allow(clippy::too_many_arguments)]
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    bind_groups: &StaticDrawGroups,
    textures: &wgpu::BindGroup,
    water_group: &wgpu::BindGroup,
    args: &[&wgpu::Buffer],
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("water"),
      color_attachments: &[
        Some(wgpu::RenderPassColorAttachment {
          view: &targets.scene,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        }),
        Some(wgpu::RenderPassColorAttachment {
          view: &targets.distortion,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: NEUTRAL_DISTORTION,
              g: NEUTRAL_DISTORTION,
              b: 0.0,
              a: NEUTRAL_DISTORTION,
            }),
            store: wgpu::StoreOp::Store,
          },
        }),
      ],
      // Read only, so the same depth is sampled for what lies behind the water.
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &targets.depth,
        depth_ops: None,
        stencil_ops: None,
      }),
      ..Default::default()
    });

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.set_bind_group(3, water_group, &[]);

    for (batch, pipeline) in StaticBatch::list_water().zip(&self.pipelines) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(1, &bind_groups.layouts[batch.layout.get_index()], &[]);

      for args in args {
        pass.draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [view_layout, scene_layout, texture_layout]: &[wgpu::BindGroupLayout; 3],
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/water")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("water"),
      bind_group_layouts: &[
        Some(view_layout),
        Some(scene_layout),
        Some(texture_layout),
        Some(layout),
      ],
      ..Default::default()
    });
    let blended = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })
    };
    let targets: [Option<wgpu::ColorTargetState>; 2] = [blended(ViewTargets::SCENE), blended(ViewTargets::DISTORTION)];

    StaticBatch::list_water()
      .map(|_| {
        create_checked(device, "water", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
              module: &module,
              entry_point: Some("vs_water"),
              compilation_options: Default::default(),
              buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
              module: &module,
              entry_point: Some("fs_water"),
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
              depth_compare: Some(wgpu::CompareFunction::Greater),
              stencil: Default::default(),
              bias: Default::default(),
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
