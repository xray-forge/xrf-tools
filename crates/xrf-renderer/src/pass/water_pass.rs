use wgpu::util::DeviceExt;
use xrf_error::XrfResult;

use crate::contract::render_water_mode::RenderWaterMode;
use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;
use crate::host::render_bundle::RenderBundle;
use crate::pass::fullscreen_pipeline::{
  begin_cleared_pass, buffer_binding, create_fullscreen_pipeline, texture_binding,
};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_batch_pipelines::WaterBatchPipelines;
use crate::pass::water_groups::WaterGroups;
use crate::pass::water_sources::WaterSources;
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

/// What each blur pass reads its source by: its direction, and the share of the source's size its target is.
const BLUR_ACROSS: [f32; 4] = [1.0, 0.0, 2.0, 0.0];
const BLUR_DOWN: [f32; 4] = [0.0, 1.0, 1.0, 0.0];

/// Draws a viewport's visible water over its lit scene, tested against the G-buffer's depth without writing it, and the
/// distortion each surface causes into the distortion target while the water distorts. Only the water nearest along
/// each pixel draws: a depth pass writes it first, as the engine's water, drawn in its index order, lets a fold of its
/// surface draw over a nearer one. The enhanced water reads the scene as it stood before the water, which it refracts,
/// and while it reflects draws its reflection first, accumulated over the frames before and blurred.
pub struct WaterPass {
  layout: wgpu::BindGroupLayout,
  depth_layout: wgpu::BindGroupLayout,
  reflection_layout: wgpu::BindGroupLayout,
  blur_layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 3],
  /// One a water batch, in `StaticBatch::list_water` order.
  pipelines: Vec<WaterBatchPipelines>,
  blur: wgpu::RenderPipeline,
  blur_sampler: wgpu::Sampler,
  blur_directions: [wgpu::Buffer; 2],
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
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let cube: wgpu::TextureViewDimension = wgpu::TextureViewDimension::Cube;
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let depth = |binding: u32| texture_entry(binding, fragment, wgpu::TextureSampleType::Depth, flat);
    let sampler = |binding: u32| wgpu::BindGroupLayoutEntry {
      binding,
      visibility: fragment,
      ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
      count: None,
    };
    // What both the surface and the reflection read of the frame.
    let shared: [wgpu::BindGroupLayoutEntry; 8] = [
      uniform_entry(0, fragment),
      uniform_entry(1, fragment),
      depth(2),
      texture_entry(3, fragment, filtered, cube),
      texture_entry(4, fragment, filtered, cube),
      sampler(5),
      depth(6),
      texture_entry(7, fragment, filtered, flat),
    ];
    let create_layout = |label: &str, extra: &[wgpu::BindGroupLayoutEntry]| {
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[shared.as_slice(), extra].concat(),
      })
    };
    let layout: wgpu::BindGroupLayout = create_layout(
      "water",
      &[
        texture_entry(8, fragment, filtered, flat),
        texture_entry(9, fragment, filtered, flat),
        texture_entry(10, fragment, filtered, flat),
        texture_entry(13, fragment, filtered, flat),
        texture_entry(14, fragment, filtered, flat),
        texture_entry(15, fragment, filtered, flat),
        texture_entry(16, fragment, filtered, flat),
        texture_entry(17, fragment, filtered, flat),
        texture_entry(18, fragment, filtered, flat),
      ],
    );
    let reflection_layout: wgpu::BindGroupLayout = create_layout(
      "water reflection",
      &[
        texture_entry(11, fragment, filtered, flat),
        texture_entry(12, fragment, filtered, flat),
      ],
    );
    let depth_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("water depth"),
      entries: &[uniform_entry(1, wgpu::ShaderStages::VERTEX)],
    });
    let blur_stages: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let blur_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("water blur"),
      entries: &[
        texture_entry(0, blur_stages, filtered, flat),
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: blur_stages,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        uniform_entry(2, blur_stages),
        uniform_entry(3, blur_stages),
      ],
    });
    let layouts: [wgpu::BindGroupLayout; 3] = [view_layout.clone(), scene_layout.clone(), texture_layout.clone()];
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
    let load = |path: &str| -> Option<wgpu::TextureView> {
      bundle
        .read_bundled(path)
        .and_then(|bytes| DecodedTexture::from_dds(&bytes))
        .inspect_err(|error| log::error!("The enhanced water reads no map '{path}': {error}"))
        .ok()
        .map(|texture| texture.upload(device, queue))
    };
    let direction = |label: &str, values: [f32; 4]| {
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&values),
        usage: wgpu::BufferUsages::UNIFORM,
      })
    };

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts, [&layout, &depth_layout, &reflection_layout])?,
      blur: Self::create_blur(device, shaders, &blur_layout)?,
      blur_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("water blur"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      blur_directions: [
        direction("water blur across", BLUR_ACROSS),
        direction("water blur down", BLUR_DOWN),
      ],
      blue_noise: load(BLUE_NOISE).unwrap_or_else(|| nothing.clone()),
      perlin: load(PERLIN).unwrap_or_else(|| nothing.clone()),
      normal: load(NORMAL).unwrap_or_else(|| nothing.clone()),
      wind: load(WIND).unwrap_or_else(|| nothing.clone()),
      caustics: load(CAUSTICS).unwrap_or_else(|| nothing.clone()),
      height: load(HEIGHT).unwrap_or_else(|| nothing.clone()),
      ripples: load(RIPPLES).unwrap_or_else(|| nothing.clone()),
      nothing,
      layouts,
      generation: shaders.get_generation(),
      layout,
      depth_layout,
      reflection_layout,
      blur_layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      let layouts = [&self.layout, &self.depth_layout, &self.reflection_layout];

      match Self::create_pipelines(device, shaders, &self.layouts, layouts)
        .and_then(|pipelines| Ok((pipelines, Self::create_blur(device, shaders, &self.blur_layout)?)))
      {
        Ok((pipelines, blur)) => {
          self.pipelines = pipelines;
          self.blur = blur;
        }
        Err(error) => log::error!("Water rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Binds what each of the water's passes reads of its frame.
  pub fn create_bind_groups(&self, device: &wgpu::Device, sources: &WaterSources<'_>) -> WaterGroups {
    let targets: &ViewTargets = sources.targets;
    let scene: &wgpu::TextureView = sources.scene.unwrap_or(&self.nothing);
    let create = |label: &str, layout: &wgpu::BindGroupLayout, extra: &[wgpu::BindGroupEntry<'_>]| {
      let shared: [wgpu::BindGroupEntry<'_>; 8] = [
        buffer_binding(0, sources.lighting),
        buffer_binding(1, sources.water),
        texture_binding(2, &targets.depth),
        texture_binding(3, sources.skies[0]),
        texture_binding(4, sources.skies[1]),
        wgpu::BindGroupEntry {
          binding: 5,
          resource: wgpu::BindingResource::Sampler(sources.sky_sampler),
        },
        texture_binding(6, &targets.water_depth),
        texture_binding(7, scene),
      ];

      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[shared.as_slice(), extra].concat(),
      })
    };
    let surface = |index: usize| {
      let (blurred, clear) = sources.reflection.map_or((&self.nothing, &self.nothing), |reflection| {
        (&reflection.blurred[1], &reflection.histories[index])
      });

      create(
        "water",
        &self.layout,
        &[
          texture_binding(8, blurred),
          texture_binding(9, clear),
          texture_binding(10, &self.perlin),
          texture_binding(13, &self.normal),
          texture_binding(14, &self.wind),
          texture_binding(15, &self.caustics),
          texture_binding(16, &targets.light),
          texture_binding(17, &self.height),
          texture_binding(18, &self.ripples),
        ],
      )
    };
    let blur = |source: &wgpu::TextureView, direction: &wgpu::Buffer| {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("water blur"),
        layout: &self.blur_layout,
        entries: &[
          texture_binding(0, source),
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&self.blur_sampler),
          },
          buffer_binding(2, sources.water),
          buffer_binding(3, direction),
        ],
      })
    };

    WaterGroups {
      depth: device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("water depth"),
        layout: &self.depth_layout,
        entries: &[buffer_binding(1, sources.water)],
      }),
      surface: [surface(0), surface(1)],
      reflection: sources.reflection.map(|reflection| {
        [0, 1].map(|index| {
          create(
            "water reflection",
            &self.reflection_layout,
            &[
              texture_binding(11, &reflection.histories[1 - index]),
              texture_binding(12, &self.blue_noise),
            ],
          )
        })
      }),
      blur: sources.reflection.map(|reflection| {
        (
          [0, 1].map(|index| blur(&reflection.histories[index], &self.blur_directions[0])),
          blur(&reflection.blurred[0], &self.blur_directions[1]),
        )
      }),
    }
  }

  /// Draws the water each argument buffer lists: the nearest surface's depth, the enhanced water's reflection and its
  /// blur while it reflects, then the surface as the mode draws it; `mark` times each but the surface, which the caller
  /// times.
  #[allow(clippy::too_many_arguments)]
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    bind_groups: &StaticDrawGroups,
    textures: &wgpu::BindGroup,
    (groups, mode, reflection): (&WaterGroups, RenderWaterMode, Option<&WaterReflection>),
    args: &[&wgpu::Buffer],
    mark: &mut dyn FnMut(&mut wgpu::CommandEncoder, &'static str),
  ) {
    {
      let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("water depth"),
        color_attachments: &[],
        // Reversed, so nothing is zero and the nearest water is the greatest.
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &targets.water_depth,
          depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(0.0),
            store: wgpu::StoreOp::Store,
          }),
          stencil_ops: None,
        }),
        ..Default::default()
      });

      self.record(&mut pass, (view, bind_groups, textures, &groups.depth), args, |it| {
        &it.depth
      });
    }

    mark(encoder, "water depth");

    let index: usize = reflection.map_or(0, |it| it.index);

    if mode == RenderWaterMode::Enhanced
      && let (Some(reflection), Some(reflection_groups), Some((across, down))) =
        (reflection, &groups.reflection, &groups.blur)
    {
      self.draw_reflection(
        encoder,
        targets,
        (view, bind_groups, textures, &reflection_groups[index]),
        &reflection.histories[index],
        args,
      );
      mark(encoder, "water reflection");
      self.draw_blur(encoder, &across[index], &reflection.blurred[0]);
      self.draw_blur(encoder, down, &reflection.blurred[1]);
      mark(encoder, "water blur");
    }

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
            load: wgpu::LoadOp::Load,
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

    self.record(
      &mut pass,
      (view, bind_groups, textures, &groups.surface[index]),
      args,
      |it| match mode {
        RenderWaterMode::Engine => &it.engine,
        RenderWaterMode::Enhanced => &it.enhanced,
      },
    );
  }

  fn draw_reflection(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    groups: (&ViewBinding, &StaticDrawGroups, &wgpu::BindGroup, &wgpu::BindGroup),
    history: &wgpu::TextureView,
    args: &[&wgpu::Buffer],
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("water reflection"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: history,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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

    self.record(&mut pass, groups, args, |it| &it.reflection);
  }

  fn draw_blur(&self, encoder: &mut wgpu::CommandEncoder, group: &wgpu::BindGroup, target: &wgpu::TextureView) {
    let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass(encoder, "water blur", target);

    pass.set_pipeline(&self.blur);
    pass.set_bind_group(0, group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn record(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    (view, bind_groups, textures, water_group): (&ViewBinding, &StaticDrawGroups, &wgpu::BindGroup, &wgpu::BindGroup),
    args: &[&wgpu::Buffer],
    pick: impl Fn(&WaterBatchPipelines) -> &wgpu::RenderPipeline,
  ) {
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.set_bind_group(3, water_group, &[]);

    for (batch, pipelines) in StaticBatch::list_water().zip(&self.pipelines) {
      pass.set_pipeline(pick(pipelines));
      pass.set_bind_group(1, &bind_groups.layouts[batch.layout.get_index()], &[]);

      for args in args {
        pass.draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  fn create_blur(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/water_blur",
      "fs_water_blur",
      &[Some(layout)],
      WaterReflection::FORMAT,
    )
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [view_layout, scene_layout, texture_layout]: &[wgpu::BindGroupLayout; 3],
    [layout, depth_layout, reflection_layout]: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<Vec<WaterBatchPipelines>> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/water")?;
    let create_layout = |label: &str, water: &wgpu::BindGroupLayout| {
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(view_layout), Some(scene_layout), Some(texture_layout), Some(water)],
        ..Default::default()
      })
    };
    let pipeline_layout: wgpu::PipelineLayout = create_layout("water", layout);
    let depth_pipeline_layout: wgpu::PipelineLayout = create_layout("water depth", depth_layout);
    let reflection_pipeline_layout: wgpu::PipelineLayout = create_layout("water reflection", reflection_layout);
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
