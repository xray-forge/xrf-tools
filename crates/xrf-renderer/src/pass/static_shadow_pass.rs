use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::overhead_shift_parameters::OverheadShiftParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::shadow_casters::ShadowCasters;
use crate::pass::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
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
  /// Restores a tile of the light atlas from the static atlas, leaving the rest of it; and shifts an overhead map that
  /// moved, from its copy.
  writers: [wgpu::RenderPipeline; 2],
  writer_layouts: [wgpu::BindGroupLayout; 2],
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
    let writer_layouts: [wgpu::BindGroupLayout; 2] = [
      ShadowTileRestoreParameters::create_layout(device),
      OverheadShiftParameters::create_layout(device),
    ];
    let (pipelines, clear, writers) =
      Self::create_pipelines(device, shaders, (view_layout, layout, texture_layout), &writer_layouts)?;

    Ok(Self {
      pipelines,
      clear,
      writers,
      writer_layouts,
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
        &self.writer_layouts,
      ) {
        Ok((pipelines, clear, writers)) => {
          self.pipelines = pipelines;
          self.clear = clear;
          self.writers = writers;
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

  /// Clears one region of the map the pass draws into (a light face's tile, a band of a cascade), or restores it from
  /// the static atlas where `restore` is given, and draws the `casters` its list holds into it, the rest of the map
  /// kept: its arguments start at `args_offset`.
  pub fn record_tile(
    &self,
    context: &mut RasterContext<'_>,
    (region, restore): ([u32; 4], Option<&ShadowTileRestoreParameters>),
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: (&wgpu::Buffer, u64),
    casters: ShadowCasters,
  ) {
    set_region(context, region);

    match restore {
      Some(restore) => {
        context.get_pass().set_pipeline(&self.writers[0]);
        context.bind(restore);
      }
      None => context.get_pass().set_pipeline(&self.clear),
    }

    context.get_pass().draw(0..3, 0..1);
    self.record_casters(context, draw, args, casters);
  }

  /// Shifts the whole of an overhead map that moved from its copy, each texel taking its old self's depth where the map
  /// covered it before.
  pub fn record_shift(&self, context: &mut RasterContext<'_>, shift: &OverheadShiftParameters) {
    context.get_pass().set_pipeline(&self.writers[1]);
    context.bind(shift);
    context.get_pass().draw(0..3, 0..1);
  }

  /// Draws the `casters` a list holds into a region of the map the pass draws into, the rest of it kept.
  pub fn record_region(
    &self,
    context: &mut RasterContext<'_>,
    region: [u32; 4],
    draw: (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &wgpu::Buffer,
    casters: ShadowCasters,
  ) {
    set_region(context, region);
    self.record_casters(context, draw, (args, 0), casters);
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
    writer_layouts: &[wgpu::BindGroupLayout; 2],
  ) -> XrfResult<(
    Vec<wgpu::RenderPipeline>,
    wgpu::RenderPipeline,
    [wgpu::RenderPipeline; 2],
  )> {
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

    let writers: [wgpu::RenderPipeline; 2] = [
      create_depth_writer(
        device,
        shaders,
        ("frame/shadow_tile_restore", "shadow tile restore"),
        ("vs_restore", "fs_restore"),
        &writer_layouts[0],
      )?,
      create_depth_writer(
        device,
        shaders,
        ("frame/overhead_shift", "overhead shift"),
        ("vs_shift", "fs_shift"),
        &writer_layouts[1],
      )?,
    ];

    Ok((pipelines, clear, writers))
  }
}

/// Draws into a region of the map a pass draws into alone: its texels across and down, its width and its height.
pub fn set_region(context: &mut RasterContext<'_>, [x, y, width, height]: [u32; 4]) {
  let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

  pass.set_viewport(x as f32, y as f32, width as f32, height as f32, 0.0, 1.0);
  pass.set_scissor_rect(x, y, width, height);
}

/// A pipeline writing a shadow map's depth over its viewport from what its bind group reads, whatever stood there.
fn create_depth_writer(
  device: &wgpu::Device,
  shaders: &ShaderLibrary,
  (module, label): (&str, &'static str),
  (vertex, fragment): (&str, &str),
  layout: &wgpu::BindGroupLayout,
) -> XrfResult<wgpu::RenderPipeline> {
  let module: wgpu::ShaderModule = create_module(device, shaders, module)?;
  let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some(label),
    bind_group_layouts: &[Some(layout)],
    ..Default::default()
  });

  create_checked(device, label, || {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some(label),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some(vertex),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &module,
        entry_point: Some(fragment),
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
  })
}
