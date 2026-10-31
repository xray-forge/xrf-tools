use std::ops::Range;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::particle_batch::ParticleBatch;
use crate::pass::particle_blend::ParticleBlend;
use crate::pass::particle_parameters::ParticleParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

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

/// The pass's pipelines, made again together whenever the source change.
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
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = ParticleParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, view_layout, &layout, texture_layout)?,
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
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout, &self.texture_layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Particles rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// The sampler its draws read a sprite through where its surface clamps, which their parameters bind.
  pub fn get_clamped_sampler(&self) -> &wgpu::Sampler {
    &self.clamped_sampler
  }

  /// Draws every batch into the scene the pass draws into, each blended as its own.
  pub fn record_colour(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ParticleParameters<'_>,
    texture_group: &wgpu::BindGroup,
    batches: &[ParticleBatch],
  ) {
    let pass: &mut wgpu::RenderPass<'static> = Self::bind(context, view, parameters, texture_group);

    for batch in batches {
      pass.set_pipeline(&self.pipelines.colour[batch.blend.get_index()]);
      pass.draw(Self::to_vertices(batch.first..batch.first + batch.count), 0..1);
    }
  }

  /// Draws the distorting sprites' runs into the distortion target the pass draws into.
  pub fn record_distortion(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ParticleParameters<'_>,
    texture_group: &wgpu::BindGroup,
    runs: &[Range<u32>],
  ) {
    let pass: &mut wgpu::RenderPass<'static> = Self::bind(context, view, parameters, texture_group);
    pass.set_pipeline(&self.pipelines.distortion);

    for run in runs {
      pass.draw(Self::to_vertices(run.clone()), 0..1);
    }
  }

  fn bind<'c>(
    context: &'c mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ParticleParameters<'_>,
    texture_group: &wgpu::BindGroup,
  ) -> &'c mut wgpu::RenderPass<'static> {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, texture_group, &[]);
    pass
  }

  /// The vertices drawing a run of quads, six a quad as `QuadIB` indexes them.
  fn to_vertices(quads: Range<u32>) -> Range<u32> {
    quads.start * 6..quads.end * 6
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<ParticlePipelines> {
    let create = |label: &'static str,
                  entry_point: &'static str,
                  format: wgpu::TextureFormat,
                  blend: wgpu::BlendState|
     -> XrfResult<wgpu::RenderPipeline> {
      // todo: Cull `dfCulling` effects' quads by `dfCullCCW` as `CParticleEffect::Render` does; both sides draw.
      source.get_render_pipeline(
        &RenderPipelineDescription::new(label, "frame/particles", "vs_particle")
          .with_fragment(entry_point)
          .with_bind_group_layouts(&[Some(view_layout), Some(layout), Some(texture_layout)])
          .with_depth_stencil(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: Default::default(),
            bias: Default::default(),
          })
          .with_target(wgpu::ColorTargetState {
            format,
            blend: Some(blend),
            write_mask: wgpu::ColorWrites::ALL,
          }),
      )
    };

    Ok(ParticlePipelines {
      colour: ParticleBlend::ALL
        .iter()
        .map(|blend| {
          let entry_point: &'static str = if blend.is_multiplied() {
            "fs_particle_multiplied"
          } else {
            "fs_particle"
          };

          create("particles", entry_point, ViewTargets::SCENE, blend.get_blend_state())
        })
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
