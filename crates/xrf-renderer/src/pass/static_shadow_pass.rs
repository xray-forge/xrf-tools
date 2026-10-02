use xrf_error::XrfResult;

use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::shadow_tile::ShadowTile;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::shader::shader_library::ShaderLibrary;

/// Draws the static scene's casters into one layer of a shadow's map, depth alone: one indirect draw a batch, a
/// view's visible clusters as instances. Both faces cast, so a surface seen edge on from the sun leaks no light.
pub struct StaticShadowPass {
  view_layout: wgpu::BindGroupLayout,
  layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  /// One a batch, by its index.
  pipelines: Vec<wgpu::RenderPipeline>,
  /// Clears a tile of an atlas to the far plane, leaving the rest of it.
  clear: wgpu::RenderPipeline,
  generation: u64,
}

impl StaticShadowPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let (pipelines, clear) = Self::create_pipelines(device, shaders, view_layout, layout, texture_layout)?;

    Ok(Self {
      pipelines,
      clear,
      view_layout: view_layout.clone(),
      layout: layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout, &self.texture_layout) {
        Ok((pipelines, clear)) => {
          self.pipelines = pipelines;
          self.clear = clear;
        }
        Err(error) => log::error!("Static shadow rejected, casting with the last one: {error}"),
      }
    }
  }

  /// Clears a cascade's layer and draws the casters its own list holds into it.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    view: &ViewBinding,
    bind_groups: &[wgpu::BindGroup; 2],
    textures: &wgpu::BindGroup,
    args: &wgpu::Buffer,
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("static shadow"),
      color_attachments: &[],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: target,
        depth_ops: Some(wgpu::Operations {
          load: wgpu::LoadOp::Clear(0.0),
          store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
      }),
      ..Default::default()
    });

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);

    for batch in StaticBatch::list() {
      pass.set_pipeline(&self.pipelines[batch.get_index() as usize]);
      pass.set_bind_group(1, &bind_groups[batch.layout.get_index()], &[]);
      pass.draw_indirect(args, batch.get_index() as u64 * 16);
    }
  }

  /// Clears one tile of an atlas and draws the casters a light face's list holds into it, the rest of the atlas kept.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_tile(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    tile: ShadowTile,
    view: &ViewBinding,
    bind_groups: &[wgpu::BindGroup; 2],
    textures: &wgpu::BindGroup,
    args: &wgpu::Buffer,
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("light shadow face"),
      color_attachments: &[],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: target,
        depth_ops: Some(wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
      }),
      ..Default::default()
    });

    pass.set_viewport(
      tile.x as f32,
      tile.y as f32,
      tile.size as f32,
      tile.size as f32,
      0.0,
      1.0,
    );
    pass.set_scissor_rect(tile.x, tile.y, tile.size, tile.size);
    pass.set_pipeline(&self.clear);
    pass.draw(0..3, 0..1);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);

    for batch in StaticBatch::list() {
      pass.set_pipeline(&self.pipelines[batch.get_index() as usize]);
      pass.set_bind_group(1, &bind_groups[batch.layout.get_index()], &[]);
      pass.draw_indirect(args, batch.get_index() as u64 * 16);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(Vec<wgpu::RenderPipeline>, wgpu::RenderPipeline)> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/gbuffer")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static shadow"),
      bind_group_layouts: &[Some(view_layout), Some(layout), Some(texture_layout)],
      ..Default::default()
    });

    let pipelines: Vec<wgpu::RenderPipeline> = StaticBatch::list()
      .map(|batch| {
        let vertex: &str = match batch.layout {
          StaticLayout::Baked => "vs_baked",
          StaticLayout::Tree => "vs_tree",
        };
        // An opaque caster writes depth alone; a cut-out one is cut first.
        let fragment: Option<wgpu::FragmentState<'_>> =
          (batch.class == StaticClass::CutOut).then(|| wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_shadow_cut_out"),
            compilation_options: Default::default(),
            targets: &[],
          });

        create_checked(device, "static shadow", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("static shadow"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
              module: &module,
              entry_point: Some(vertex),
              compilation_options: Default::default(),
              buffers: &[],
            },
            fragment: fragment.clone(),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
              format: SunShadowMaps::FORMAT,
              depth_write_enabled: Some(true),
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
      .collect::<XrfResult<_>>()?;
    let clear_module: wgpu::ShaderModule = create_module(device, shaders, "frame/depth_clear")?;
    let clear_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&Default::default());
    let clear: wgpu::RenderPipeline = create_checked(device, "shadow tile clear", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow tile clear"),
        layout: Some(&clear_layout),
        vertex: wgpu::VertexState {
          module: &clear_module,
          entry_point: Some("vs_clear"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: None,
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
          format: SunShadowMaps::FORMAT,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Always),
          stencil: Default::default(),
          bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    })?;

    Ok((pipelines, clear))
  }
}
