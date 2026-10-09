use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::reflection_history::ReflectionHistory;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Stochastic screen-space reflections at the size their quality traces at: each glossy pixel's ray scattered by its
/// roughness and walked up and down the frame's nearest depth pyramid, what it met lit; then denoised, the last frame's
/// reflection reprojected, an eighth-size average taken, filtered across the surface and resolved over time into the
/// view's history, which combine reads.
pub struct ReflectionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipelines: ReflectionPipelines,
  /// What a stage reads in place of a history or a stage before it where there is none: one texel of nothing.
  empty: wgpu::TextureView,
  /// What combine and the debug view read where nothing is traced: one texel marked untraced.
  untraced: wgpu::TextureView,
  /// The history's bilinear reads.
  sampler: wgpu::Sampler,
  generation: u64,
}

/// The trace and the denoiser's four stages.
struct ReflectionPipelines {
  trace: wgpu::RenderPipeline,
  reproject: wgpu::RenderPipeline,
  average: wgpu::RenderPipeline,
  prefilter: wgpu::RenderPipeline,
  resolve: wgpu::RenderPipeline,
}

impl ReflectionPass {
  /// The denoiser's stages, by their entry points.
  pub const REPROJECT: &'static str = "fs_reproject";
  pub const AVERAGE_STAGE: &'static str = "fs_average";
  pub const PREFILTER: &'static str = "fs_prefilter";
  pub const RESOLVE: &'static str = "fs_resolve";
  /// What the trace writes: what is reflected, then the ray's length in metres, below none where nothing is traced.
  pub const TRACED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// The reprojected history, then how many frames it holds; and its variance.
  pub const REPROJECTED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const VARIANCE: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
  /// The eighth-size average, and the prefiltered reflection, then its variance.
  pub const AVERAGE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const PREFILTERED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Traced pixels a side each texel of the average stands for.
  pub const AVERAGE_RATIO: u32 = 8;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = ReflectionParameters::create_layout(device);
    // Half floats: nothing, and nothing with an alpha of minus one.
    let texel = |label: &str, alpha: u16| -> wgpu::TextureView {
      device
        .create_texture_with_data(
          queue,
          &wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
              width: 1,
              height: 1,
              depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::TRACED,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
          },
          Default::default(),
          bytemuck::cast_slice(&[0u16, 0, 0, alpha]),
        )
        .create_view(&Default::default())
    };

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, (view_layout, &layout, sky_layout))?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      empty: texel("reflections empty", 0),
      untraced: texel("reflections none", 0xbc00),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("reflection history"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, (&self.view_layout, &self.layout, &self.sky_layout)) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Reflections rejected, tracing with the last ones: {error}"),
      }
    }
  }

  /// The texel a stage reads where no history or stage before it is bound.
  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// The texel combine and the debug view read where nothing is traced.
  pub fn get_untraced(&self) -> &wgpu::TextureView {
    &self.untraced
  }

  /// The history's bilinear sampler.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Traces every reflecting pixel's ray into the targets the pass draws into.
  pub fn record_trace(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
    sky: &SkyParameters<'_>,
  ) {
    context.bind(parameters);
    context.bind(sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.trace);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  /// Draws one of the denoiser's stages into the targets the pass draws into.
  pub fn record_stage(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
    stage: &str,
  ) {
    let pipeline: &wgpu::RenderPipeline = match stage {
      Self::REPROJECT => &self.pipelines.reproject,
      Self::AVERAGE_STAGE => &self.pipelines.average,
      Self::PREFILTER => &self.pipelines.prefilter,
      _ => &self.pipelines.resolve,
    };

    Self::draw(context, pipeline, view, parameters);
  }

  fn draw(
    context: &mut RasterContext<'_>,
    pipeline: &wgpu::RenderPipeline,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    (view_layout, layout, sky_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
  ) -> XrfResult<ReflectionPipelines> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/reflections")?;
    let traced_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("reflection trace"),
      bind_group_layouts: &[Some(view_layout), Some(layout), Some(sky_layout)],
      ..Default::default()
    });
    let stage_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("reflections"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    let create = |pipeline_layout: &wgpu::PipelineLayout,
                  fragment: &str,
                  formats: &[wgpu::TextureFormat]|
     -> XrfResult<wgpu::RenderPipeline> {
      let targets: Vec<Option<wgpu::ColorTargetState>> = formats.iter().map(|it| Some((*it).into())).collect();

      create_checked(device, fragment, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some(fragment),
          layout: Some(pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets: &targets,
          }),
          primitive: Default::default(),
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };

    Ok(ReflectionPipelines {
      trace: create(&traced_layout, "fs_trace", &[Self::TRACED])?,
      reproject: create(&stage_layout, Self::REPROJECT, &[Self::REPROJECTED, Self::VARIANCE])?,
      average: create(&stage_layout, Self::AVERAGE_STAGE, &[Self::AVERAGE])?,
      prefilter: create(&stage_layout, Self::PREFILTER, &[Self::PREFILTERED])?,
      resolve: create(
        &stage_layout,
        Self::RESOLVE,
        &[
          ReflectionHistory::RADIANCE_FORMAT,
          ReflectionHistory::SURFACE_FORMAT,
          ReflectionHistory::HELD_FORMAT,
        ],
      )?,
    })
  }
}
