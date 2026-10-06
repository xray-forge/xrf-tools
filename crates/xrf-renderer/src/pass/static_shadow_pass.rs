use xrf_error::XrfResult;
use xrf_renderer_core::RasterContext;

use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::shadow_tile::ShadowTile;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::shader::shader_library::ShaderLibrary;

/// The override constants every shadow draw is built with: it leaves out the surfaces casting none.
const SHADOW_DRAW: &[(&str, f64)] = &[("IS_SHADOW_DRAW", 1.0)];

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

  /// Draws the casters a shadow's own list holds into the map the pass draws into, cleared to the far plane.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &wgpu::Buffer,
  ) {
    self.record_casters(context, draw, (args, 0), true);
  }

  /// Draws the casters a list holds, leaving the trees out where `is_tree_drawn` is not set.
  pub fn record_layered(
    &self,
    context: &mut RasterContext<'_>,
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &wgpu::Buffer,
    is_tree_drawn: bool,
  ) {
    self.record_casters(context, draw, (args, 0), is_tree_drawn);
  }

  /// Clears one tile of the atlas the pass draws into and draws the casters a light face's list holds into it, the
  /// rest of the atlas kept: its arguments start at `args_offset`.
  pub fn record_tile(
    &self,
    context: &mut RasterContext<'_>,
    tile: ShadowTile,
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: (&wgpu::Buffer, u64),
  ) {
    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

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
    self.record_casters(context, draw, args, true);
  }

  /// Draws the casters a list holds, each batch's arguments from `args_offset`, the trees only where `is_tree_drawn`.
  fn record_casters(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (args, args_offset): (&wgpu::Buffer, u64),
    is_tree_drawn: bool,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list_deferred().zip(&self.pipelines) {
      if !is_tree_drawn && batch.layout == StaticLayout::Tree {
        continue;
      }

      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);
      context
        .get_pass()
        .draw_indirect(args, args_offset + batch.get_index() as u64 * 16);
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
      bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(layout)],
      ..Default::default()
    });

    let pipelines: Vec<wgpu::RenderPipeline> = StaticBatch::list_deferred()
      .map(|batch| {
        let vertex: &str = batch.layout.get_vertex_entry();
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
              compilation_options: wgpu::PipelineCompilationOptions {
                constants: SHADOW_DRAW,
                ..Default::default()
              },
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
