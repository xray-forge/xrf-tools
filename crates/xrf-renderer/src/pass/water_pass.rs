use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureAccess, GraphTextureDescriptor, PassParameters, RasterPassBuilder, UniformBinding,
};

use crate::contract::render_water_mode::RenderWaterMode;
use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;
use crate::host::render_bundle::RenderBundle;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_batch_pipelines::WaterBatchPipelines;
use crate::pass::water_blur_parameters::WaterBlurParameters;
use crate::pass::water_blur_uniform::WaterBlurUniform;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_draw::WaterDraw;
use crate::pass::water_reflection_parameters::WaterReflectionParameters;
use crate::pass::water_surface_parameters::WaterSurfaceParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::shader::shader_library::ShaderLibrary;

/// The enhanced water's maps, as the renderer's bundle keeps them: Screen Space Shaders' `fx\blue_noise`,
/// `water\water_perlin`, `fx\water_normal`, `fx\water_wind`, `fx\water_caustics`, `fx\water_height` and
/// `fx\water_sbumpvolume`.
const BLUE_NOISE: &str = "water/blue_noise.dds";
const PERLIN: &str = "water/perlin.dds";
const NORMAL: &str = "water/normal.dds";
const WIND: &str = "water/wind.dds";
const CAUSTICS: &str = "water/caustics.dds";
const HEIGHT: &str = "water/height.dds";
const RIPPLES: &str = "water/ripples.dds";

/// What every water batch binds below its pass's own group: the view, the bindless textures, and the static draws.
type WaterGroups<'a> = (&'a ViewBinding, &'a wgpu::BindGroup, &'a StaticDrawGroups);

/// Draws a viewport's visible water over its lit scene, tested against the G-buffer's depth without writing it, and the
/// distortion each surface causes into the distortion target while the water distorts. Only the water nearest along
/// each pixel draws: a depth pass writes it first, as the engine's water, drawn in its index order, lets a fold of its
/// surface draw over a nearer one. The enhanced water reads the scene as it stood before the water, which it refracts,
/// and while it reflects draws its reflection first, accumulated over the frames before and blurred.
pub struct WaterPass {
  /// The view's, the bindless textures' and the static draws' layouts, which every water pipeline binds below its own.
  layouts: [wgpu::BindGroupLayout; 3],
  /// One a water batch, in `StaticBatch::list_water` order.
  pipelines: Vec<WaterBatchPipelines>,
  blur: wgpu::RenderPipeline,
  blur_sampler: wgpu::Sampler,
  blue_noise: wgpu::TextureView,
  perlin: wgpu::TextureView,
  normal: wgpu::TextureView,
  wind: wgpu::TextureView,
  caustics: wgpu::TextureView,
  height: wgpu::TextureView,
  ripples: wgpu::TextureView,
  /// What a binding a pass does not read is given: a texel of nothing.
  nothing: wgpu::TextureView,
  generation: u64,
}

impl WaterPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    [view_layout, scene_layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
    let layouts: [wgpu::BindGroupLayout; 3] = [view_layout.clone(), texture_layout.clone(), scene_layout.clone()];
    let nothing: wgpu::TextureView = device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("water nothing"),
        size: wgpu::Extent3d {
          width: 1,
          height: 1,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ViewTargets::SCENE,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default());
    // A map the bundle cannot give is nothing: the reflection's march unjittered, its blur unmixed, the waves flat.
    let load = |path: &str| -> wgpu::TextureView {
      bundle
        .read_bundled(path)
        .and_then(|bytes| DecodedTexture::from_dds(&bytes))
        .inspect_err(|error| log::error!("The enhanced water reads no map '{path}': {error}"))
        .ok()
        .map_or_else(|| nothing.clone(), |texture| texture.upload(device, queue))
    };

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts)?,
      blur: Self::create_blur(device, shaders)?,
      blur_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("water blur"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      blue_noise: load(BLUE_NOISE),
      perlin: load(PERLIN),
      normal: load(NORMAL),
      wind: load(WIND),
      caustics: load(CAUSTICS),
      height: load(HEIGHT),
      ripples: load(RIPPLES),
      nothing,
      layouts,
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layouts)
        .and_then(|pipelines| Ok((pipelines, Self::create_blur(device, shaders)?)))
      {
        Ok((pipelines, blur)) => {
          self.pipelines = pipelines;
          self.blur = blur;
        }
        Err(error) => log::error!("Water rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Declares the frame's water: the scene copied for the enhanced water to refract, the nearest surface's depth, the
  /// enhanced water's reflection and its blur while it reflects, then the surface as the mode draws it.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    runtime: &mut GraphRuntime,
    draw: WaterDraw<'a>,
  ) {
    let water = draw.water;
    let (width, height) = draw.targets.size;
    let uniform: UniformBinding<WaterUniform> = runtime.push_uniform(water.get_uniform());
    let mut import = |label: &'static str, view: &'a wgpu::TextureView| bindings.import_view(graph, label, view);
    let nothing: GraphTexture = import("water nothing", &self.nothing);
    let skies: [GraphTexture; 2] = [import("sky cube 0", draw.skies[0]), import("sky cube 1", draw.skies[1])];
    let maps: [GraphTexture; 6] = [
      import("water perlin", &self.perlin),
      import("water normal", &self.normal),
      import("water wind", &self.wind),
      import("water caustics", &self.caustics),
      import("water height", &self.height),
      import("water ripples", &self.ripples),
    ];
    let reflection: Option<(&WaterReflection, [GraphTexture; 2], GraphTexture)> =
      water.get_reflection().map(|reflection| {
        (
          reflection,
          [
            import("water reflection 0", &reflection.histories[0]),
            import("water reflection 1", &reflection.histories[1]),
          ],
          import("water blue noise", &self.blue_noise),
        )
      });
    let args: Vec<GraphBuffer> = draw
      .args
      .iter()
      .map(|args| bindings.import_buffer(graph, "static draw arguments", args))
      .collect();
    let groups: WaterGroups<'a> = (draw.view, draw.textures, draw.draw_groups);

    let scene: GraphTexture = if water.is_refracting() {
      let copy: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "water scene",
        width,
        height,
        ViewTargets::SCENE,
      ));
      let source: GraphTexture = draw.targets.scene;

      graph
        .add_encoder_pass("water copy")
        .texture(source, GraphTextureAccess::CopySource)
        .texture(copy, GraphTextureAccess::CopyDestination)
        .record(move |context| {
          let (source, copy) = (context.get_texture(source), context.get_texture(copy));

          context.get_encoder().copy_texture_to_texture(
            source.texture.as_image_copy(),
            copy.texture.as_image_copy(),
            copy.texture.size(),
          );
        });

      copy
    } else {
      nothing
    };

    let nearest: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "water depth",
      width,
      height,
      ViewTargets::DEPTH,
    ));
    let depth_parameters: WaterDepthParameters = WaterDepthParameters { water: uniform };

    self.add_batch_pass(
      graph
        .add_raster_pass("water depth")
        // Reversed, so nothing is zero and the nearest water is the greatest.
        .depth(GraphDepthAttachment::new(nearest, wgpu::LoadOp::Clear(0.0))),
      depth_parameters,
      (&args, groups),
      |it| &it.depth,
    );

    let (blurred, clear): (GraphTexture, GraphTexture) = match reflection {
      Some((reflection, histories, blue_noise)) => {
        let index: usize = reflection.index;
        let reflection_parameters: WaterReflectionParameters<'a> = WaterReflectionParameters {
          lighting: draw.lighting,
          water: uniform,
          depth_target: draw.targets.depth,
          sky_cube_0: skies[0],
          sky_cube_1: skies[1],
          sky_clamp: draw.sky_sampler,
          nearest_water: nearest,
          water_scene: scene,
          reflection_history: histories[1 - index],
          blue_noise,
        };

        self.add_batch_pass(
          graph
            .add_raster_pass("water reflection")
            .color(GraphColorAttachment::new(
              histories[index],
              wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            ))
            .depth(GraphDepthAttachment::new_read_only(draw.targets.depth)),
          reflection_parameters,
          (&args, groups),
          |it| &it.reflection,
        );

        let half: (u32, u32) = (width.div_ceil(2), height.div_ceil(2));
        let across: GraphTexture = self.add_blur(
          graph,
          runtime,
          ("water blur across", histories[index], half, WaterBlurUniform::ACROSS),
          uniform,
        );
        let down: GraphTexture = self.add_blur(
          graph,
          runtime,
          ("water blur down", across, half, WaterBlurUniform::DOWN),
          uniform,
        );

        (down, histories[index])
      }
      None => (nothing, nothing),
    };

    let surface_parameters: WaterSurfaceParameters<'a> = WaterSurfaceParameters {
      lighting: draw.lighting,
      water: uniform,
      depth_target: draw.targets.depth,
      sky_cube_0: skies[0],
      sky_cube_1: skies[1],
      sky_clamp: draw.sky_sampler,
      nearest_water: nearest,
      water_scene: scene,
      reflection_blurred: blurred,
      reflection_clear: clear,
      perlin_map: maps[0],
      wave_map: maps[1],
      wind_map: maps[2],
      caustics_map: maps[3],
      light_target: draw.targets.light,
      height_map: maps[4],
      ripple_map: maps[5],
    };
    let mode: RenderWaterMode = water.get_mode();

    self.add_batch_pass(
      graph
        .add_raster_pass("water")
        .color(GraphColorAttachment::new(draw.targets.scene, wgpu::LoadOp::Load))
        .color(GraphColorAttachment::new(draw.targets.distortion, wgpu::LoadOp::Load))
        // Read only, so the same depth is sampled for what lies behind the water.
        .depth(GraphDepthAttachment::new_read_only(draw.targets.depth)),
      surface_parameters,
      (&args, groups),
      move |it| match mode {
        RenderWaterMode::Engine => &it.engine,
        RenderWaterMode::Enhanced => &it.enhanced,
      },
    );
  }

  /// Declares one way of the reflection's blur, from `source` into a transient of `size`, and answers that transient.
  fn add_blur<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    runtime: &mut GraphRuntime,
    (name, source, (width, height), way): (&'static str, GraphTexture, (u32, u32), WaterBlurUniform),
    water: UniformBinding<WaterUniform>,
  ) -> GraphTexture {
    let target: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      name,
      width,
      height,
      WaterReflection::FORMAT,
    ));
    let parameters: WaterBlurParameters<'a> = WaterBlurParameters {
      source,
      source_sampler: &self.blur_sampler,
      water,
      blur: runtime.push_uniform(&way),
    };

    graph
      .add_raster_pass(name)
      .color(GraphColorAttachment::new(
        target,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .parameters(&parameters)
      .record(move |context| {
        context.bind(&parameters);
        context.get_pass().set_pipeline(&self.blur);
        context.get_pass().draw(0..3, 0..1);
      });

    target
  }

  /// Adds a pass drawing every water batch from the cull's argument buffers, with its parameters bound and the pipeline
  /// `pick` chooses.
  fn add_batch_pass<'a, P: PassParameters + Send + 'a>(
    &'a self,
    builder: RasterPassBuilder<'_, 'a>,
    parameters: P,
    (args, groups): (&[GraphBuffer], WaterGroups<'a>),
    pick: impl Fn(&WaterBatchPipelines) -> &wgpu::RenderPipeline + Send + 'a,
  ) {
    let args: Vec<GraphBuffer> = args.to_vec();

    args
      .iter()
      .fold(builder.parameters(&parameters), |builder, args| {
        builder.buffer(*args, GraphBufferAccess::Indirect)
      })
      .record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        context.bind(&parameters);
        self.draw_batches(context.get_pass(), groups, &args, pick);
      });
  }

  /// Draws every water batch each argument buffer lists with the pipeline `pick` chooses, the pass's own group bound.
  fn draw_batches(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    (view, textures, draw_groups): WaterGroups<'_>,
    args: &[&wgpu::Buffer],
    pick: impl Fn(&WaterBatchPipelines) -> &wgpu::RenderPipeline,
  ) {
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);

    for (batch, pipelines) in StaticBatch::list_water().zip(&self.pipelines) {
      pass.set_pipeline(pick(pipelines));
      pass.set_bind_group(2, &draw_groups.layouts[batch.layout.get_index()], &[]);

      for args in args {
        pass.draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  fn create_blur(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/water_blur",
      "fs_water_blur",
      &[Some(&WaterBlurParameters::create_layout(device))],
      WaterReflection::FORMAT,
    )
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [view_layout, texture_layout, scene_layout]: &[wgpu::BindGroupLayout; 3],
  ) -> XrfResult<Vec<WaterBatchPipelines>> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/water")?;
    let create_layout = |label: &str, water: &wgpu::BindGroupLayout| {
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(scene_layout), Some(water)],
        ..Default::default()
      })
    };
    let pipeline_layout: wgpu::PipelineLayout = create_layout("water", &WaterSurfaceParameters::create_layout(device));
    let depth_pipeline_layout: wgpu::PipelineLayout =
      create_layout("water depth", &WaterDepthParameters::create_layout(device));
    let reflection_pipeline_layout: wgpu::PipelineLayout =
      create_layout("water reflection", &WaterReflectionParameters::create_layout(device));
    let blended = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })
    };
    let targets: [Option<wgpu::ColorTargetState>; 2] = [blended(ViewTargets::SCENE), blended(ViewTargets::DISTORTION)];
    let reflection_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(WaterReflection::FORMAT.into())];
    let vertex = wgpu::VertexState {
      module: &module,
      entry_point: Some("vs_water"),
      compilation_options: Default::default(),
      buffers: &[],
    };
    let primitive = wgpu::PrimitiveState {
      front_face: wgpu::FrontFace::Ccw,
      cull_mode: Some(wgpu::Face::Back),
      ..Default::default()
    };
    let depth_state = |is_written: bool| wgpu::DepthStencilState {
      format: ViewTargets::DEPTH,
      depth_write_enabled: Some(is_written),
      depth_compare: Some(wgpu::CompareFunction::Greater),
      stencil: Default::default(),
      bias: Default::default(),
    };
    let surface =
      |label: &str, layout: &wgpu::PipelineLayout, entry: &str, targets: &[Option<wgpu::ColorTargetState>]| {
        create_checked(device, label, || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: vertex.clone(),
            fragment: Some(wgpu::FragmentState {
              module: &module,
              entry_point: Some(entry),
              compilation_options: Default::default(),
              targets,
            }),
            primitive,
            depth_stencil: Some(depth_state(false)),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
          })
        })
      };

    StaticBatch::list_water()
      .map(|_| {
        let depth: wgpu::RenderPipeline = create_checked(device, "water depth", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water depth"),
            layout: Some(&depth_pipeline_layout),
            vertex: vertex.clone(),
            fragment: None,
            primitive,
            depth_stencil: Some(depth_state(true)),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
          })
        })?;

        Ok(WaterBatchPipelines {
          depth,
          reflection: surface(
            "water reflection",
            &reflection_pipeline_layout,
            "fs_water_reflection",
            &reflection_targets,
          )?,
          engine: surface("water", &pipeline_layout, "fs_water", &targets)?,
          enhanced: surface("water", &pipeline_layout, "fs_water_enhanced", &targets)?,
        })
      })
      .collect()
  }
}
