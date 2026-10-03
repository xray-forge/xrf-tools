use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::shader::shader_library::ShaderLibrary;

/// What draws a strike: its model, then its top glow, then its middle one, as `shaders/frame/thunder.wgsl` names them.
const ENTRIES: [&str; 3] = ["vs_model", "vs_glow_top", "vs_glow_center"];

/// The blends a bolt's shader composites by, as `XraySurfaceDraw` names them: blended, added, alpha added, multiplied,
/// multiplied twice over.
const BLENDS: [(wgpu::BlendFactor, wgpu::BlendFactor); 5] = [
  (wgpu::BlendFactor::SrcAlpha, wgpu::BlendFactor::OneMinusSrcAlpha),
  (wgpu::BlendFactor::One, wgpu::BlendFactor::One),
  (wgpu::BlendFactor::SrcAlpha, wgpu::BlendFactor::One),
  (wgpu::BlendFactor::Dst, wgpu::BlendFactor::Zero),
  (wgpu::BlendFactor::Dst, wgpu::BlendFactor::Src),
];

/// Draws the bolt striking over a viewport's finished scene, as `dxThunderboltRender` does after the rain: its model and
/// its two glows, each composited as its shader says, tested against the G-buffer's depth without writing it.
pub struct ThunderPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// By `ENTRIES`, then by `BLENDS`.
  pipelines: Vec<wgpu::RenderPipeline>,
  generation: u64,
}

impl ThunderPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let stages: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("thunder"),
      entries: &[
        uniform_entry(0, stages),
        texture_entry(
          1,
          stages,
          wgpu::TextureSampleType::Float { filterable: true },
          wgpu::TextureViewDimension::D2,
        ),
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: stages,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        storage_entry(3, stages, false),
        storage_entry(4, stages, false),
      ],
    });

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("thunder"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
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
        Err(error) => log::error!("Thunder rejected, striking with the last one: {error}"),
      }
    }
  }

  /// One draw's bind group: the strike's uniform, the texture it draws with, and the model it places.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    uniform: &wgpu::Buffer,
    texture: &wgpu::TextureView,
    model: &WeatherModelBuffers,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("thunder"),
      layout: &self.layout,
      entries: &[
        buffer_binding(0, uniform),
        texture_binding(1, texture),
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
        buffer_binding(3, &model.vertices),
        buffer_binding(4, &model.indices),
      ],
    })
  }

  /// Draws the strike: its model's indices, when it has a model, then both glows, each by its own blend.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    groups: &[wgpu::BindGroup; 3],
    (draws, model_indices): ([XraySurfaceDraw; 3], u32),
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("thunder"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &targets.scene,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &targets.depth,
        depth_ops: None,
        stencil_ops: None,
      }),
      ..Default::default()
    });

    pass.set_bind_group(0, &view.bind_group, &[]);

    for (entry, (group, draw)) in groups.iter().zip(draws).enumerate() {
      let vertices: u32 = if entry == 0 { model_indices } else { 6 };

      if vertices == 0 {
        continue;
      }

      pass.set_pipeline(&self.pipelines[entry * BLENDS.len() + to_blend(draw)]);
      pass.set_bind_group(1, group, &[]);
      pass.draw(0..vertices, 0..1);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/thunder")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("thunder"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });

    ENTRIES
      .iter()
      .flat_map(|entry| BLENDS.iter().map(move |blend| (*entry, *blend)))
      .map(|(entry, (source, destination))| {
        let component: wgpu::BlendComponent = wgpu::BlendComponent {
          src_factor: source,
          dst_factor: destination,
          operation: wgpu::BlendOperation::Add,
        };
        let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
          format: ViewTargets::SCENE,
          blend: Some(wgpu::BlendState {
            color: component,
            alpha: component,
          }),
          write_mask: wgpu::ColorWrites::ALL,
        })];

        create_checked(device, "thunder", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("thunder"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
              module: &module,
              entry_point: Some(entry),
              compilation_options: Default::default(),
              buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
              module: &module,
              entry_point: Some("fs_thunder"),
              compilation_options: Default::default(),
              targets: &targets,
            }),
            // Both sides, `CULL_NONE`, and a glow faces the view anyway.
            primitive: Default::default(),
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

/// A shader's blend among `BLENDS`; one that composites nothing still lights the air, as the engine's lightning is
/// added.
fn to_blend(draw: XraySurfaceDraw) -> usize {
  match draw {
    XraySurfaceDraw::Blended { .. } => 0,
    XraySurfaceDraw::Added { is_weighted: true, .. } => 2,
    XraySurfaceDraw::Multiplied { is_doubled: false } => 3,
    XraySurfaceDraw::Multiplied { is_doubled: true } => 4,
    _ => 1,
  }
}
