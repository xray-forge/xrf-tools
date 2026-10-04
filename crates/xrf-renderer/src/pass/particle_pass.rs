use std::ops::Range;

use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::particle_batch::ParticleBatch;
use crate::pass::particle_blend::ParticleBlend;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Draws a level's particle sprites over its finished scene after the composited surfaces, as `render_forward` draws
/// them after combine: each run of quads with its effect's equation, tested against the scene's depth without writing
/// it and reading it to fade where they meet it; then the distorting effects' quads into the distortion target, as
/// `mapDistort` draws their `l_special` passes.
pub struct ParticlePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  clamped_sampler: wgpu::Sampler,
  pipelines: ParticlePipelines,
  generation: u64,
}

/// The pass's pipelines, made again together whenever the shaders change.
struct ParticlePipelines {
  /// One a blend, in [`ParticleBlend::ALL`]'s order.
  colour: Vec<wgpu::RenderPipeline>,
  /// `particle_distort`'s, blended over the distortion target by its alpha.
  distortion: wgpu::RenderPipeline,
}

impl ParticlePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let stages: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("particles"),
      entries: &[
        storage_entry(0, stages, false),
        storage_entry(1, stages, false),
        uniform_entry(2, stages),
        texture_entry(
          3,
          stages,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2,
        ),
        wgpu::BindGroupLayoutEntry {
          binding: 4,
          visibility: stages,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layout, texture_layout)?,
      // As the textures' own sampler, clamped to the edge as `Texture clamp` addresses a sprite.
      clamped_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("particles clamped"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: 8,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout, &self.texture_layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Particles rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// What the draws read: the quads' corners, the effects' surfaces, the frame's lighting and the scene's depth.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    vertices: &wgpu::Buffer,
    surfaces: &wgpu::Buffer,
    lighting: &wgpu::Buffer,
    targets: &ViewTargets,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("particles"),
      layout: &self.layout,
      entries: &[
        buffer_binding(0, vertices),
        buffer_binding(1, surfaces),
        buffer_binding(2, lighting),
        texture_binding(3, &targets.depth),
        wgpu::BindGroupEntry {
          binding: 4,
          resource: wgpu::BindingResource::Sampler(&self.clamped_sampler),
        },
      ],
    })
  }

  /// Draws each run of quads in order over the scene, with its equation's pipeline.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    groups: (&ViewBinding, &wgpu::BindGroup, &wgpu::BindGroup),
    batches: &[ParticleBatch],
  ) {
    let mut pass: wgpu::RenderPass<'_> = Self::begin(encoder, "particles", &targets.scene, targets, groups);

    for batch in batches {
      pass.set_pipeline(&self.pipelines.colour[batch.blend.get_index()]);
      pass.draw(Self::to_vertices(batch.first..batch.first + batch.count), 0..1);
    }
  }

  /// Draws each run of quads, back to front, into the distortion target.
  pub fn draw_distortion(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    groups: (&ViewBinding, &wgpu::BindGroup, &wgpu::BindGroup),
    runs: &[Range<u32>],
  ) {
    let mut pass: wgpu::RenderPass<'_> =
      Self::begin(encoder, "particle distortion", &targets.distortion, targets, groups);

    pass.set_pipeline(&self.pipelines.distortion);

    for run in runs {
      pass.draw(Self::to_vertices(run.clone()), 0..1);
    }
  }

  /// A pass drawing into one target, tested against the scene's depth, which stays read only so it is sampled too.
  fn begin<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    label: &str,
    target: &wgpu::TextureView,
    targets: &ViewTargets,
    (view, bind_group, texture_group): (&ViewBinding, &wgpu::BindGroup, &wgpu::BindGroup),
  ) -> wgpu::RenderPass<'a> {
    let mut pass: wgpu::RenderPass<'a> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some(label),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: target,
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
    pass.set_bind_group(1, bind_group, &[]);
    pass.set_bind_group(2, texture_group, &[]);

    pass
  }

  /// The vertices drawing a run of quads, six a quad as `QuadIB` indexes them.
  fn to_vertices(quads: Range<u32>) -> Range<u32> {
    quads.start * 6..quads.end * 6
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<ParticlePipelines> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/particles")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("particles"),
      bind_group_layouts: &[Some(view_layout), Some(layout), Some(texture_layout)],
      ..Default::default()
    });
    let create = |label: &str, entry_point: &str, format: wgpu::TextureFormat, blend: wgpu::BlendState| {
      let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
        format,
        blend: Some(blend),
        write_mask: wgpu::ColorWrites::ALL,
      })];

      create_checked(device, label, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some(label),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_particle"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            targets: &targets,
          }),
          // todo: Cull `dfCulling` effects' quads by `dfCullCCW` as `CParticleEffect::Render` does; both sides draw.
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
    };

    Ok(ParticlePipelines {
      colour: ParticleBlend::ALL
        .iter()
        .map(|blend| create("particles", "fs_particle", ViewTargets::SCENE, blend.get_blend_state()))
        .collect::<XrfResult<_>>()?,
      // `blend(true, blend.srcalpha, blend.invsrcalpha)`, as the water blends what it writes there.
      distortion: create(
        "particle distortion",
        "fs_distort",
        ViewTargets::DISTORTION,
        wgpu::BlendState::ALPHA_BLENDING,
      )?,
    })
  }
}
