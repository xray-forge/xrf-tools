use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::shadow_casters::ShadowCasters;
use crate::pass::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
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
  /// Restores a tile of the light atlas from the static atlas, leaving the rest of it.
  restore: wgpu::RenderPipeline,
  restore_layout: wgpu::BindGroupLayout,
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
    let restore_layout: wgpu::BindGroupLayout = ShadowTileRestoreParameters::create_layout(device);
    let (pipelines, clear, restore) =
      Self::create_pipelines(device, shaders, (view_layout, layout, texture_layout), &restore_layout)?;

    Ok(Self {
      pipelines,
      clear,
      restore,
      restore_layout,
      view_layout: view_layout.clone(),
      layout: layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(
        device,
        shaders,
        (&self.view_layout, &self.layout, &self.texture_layout),
        &self.restore_layout,
      ) {
        Ok((pipelines, clear, restore)) => {
          self.pipelines = pipelines;
          self.clear = clear;
          self.restore = restore;
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
    self.record_casters(context, draw, (args, 0), ShadowCasters::All);
  }

  /// Draws the `casters` a list holds of the kinds asked for.
  pub fn record_some(
    &self,
    context: &mut RasterContext<'_>,
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &wgpu::Buffer,
    casters: ShadowCasters,
  ) {
    self.record_casters(context, draw, (args, 0), casters);
  }

  /// Clears one tile of the atlas the pass draws into, or restores it from the static atlas where `restore` is given,
  /// and draws the `casters` a light face's list holds into it, the rest of the atlas kept: its arguments start at
  /// `args_offset`.
  pub fn record_tile(
    &self,
    context: &mut RasterContext<'_>,
    (tile, restore): (ShadowTile, Option<&ShadowTileRestoreParameters>),
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: (&wgpu::Buffer, u64),
    casters: ShadowCasters,
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

    match restore {
      Some(restore) => {
        context.get_pass().set_pipeline(&self.restore);
        context.bind(restore);
      }
      None => context.get_pass().set_pipeline(&self.clear),
    }

    context.get_pass().draw(0..3, 0..1);
    self.record_casters(context, draw, args, casters);
  }

  /// Draws the `casters` a list holds of the kinds asked for, each batch's arguments from `args_offset`.
  fn record_casters(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (args, args_offset): (&wgpu::Buffer, u64),
    casters: ShadowCasters,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list_deferred().zip(&self.pipelines) {
      if !casters.has(batch.layout) {
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
    (view_layout, layout, texture_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
    restore_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(Vec<wgpu::RenderPipeline>, wgpu::RenderPipeline, wgpu::RenderPipeline)> {
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

    let restore_module: wgpu::ShaderModule = create_module(device, shaders, "frame/shadow_tile_restore")?;
    let restore_pipeline_layout: wgpu::PipelineLayout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow tile restore"),
        bind_group_layouts: &[Some(restore_layout)],
        ..Default::default()
      });
    let restore: wgpu::RenderPipeline = create_checked(device, "shadow tile restore", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow tile restore"),
        layout: Some(&restore_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &restore_module,
          entry_point: Some("vs_restore"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &restore_module,
          entry_point: Some("fs_restore"),
          compilation_options: Default::default(),
          targets: &[],
        }),
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

    Ok((pipelines, clear, restore))
  }
}
