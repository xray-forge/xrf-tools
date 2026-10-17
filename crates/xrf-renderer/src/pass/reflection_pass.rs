use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_request_parameters::TraceRequestParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::reflection_history::ReflectionHistory;

/// Stochastic screen-space reflections at the size their quality traces at: each glossy pixel's ray scattered by its
/// roughness and walked up and down the frame's nearest depth pyramid, what it met lit; then denoised, the last frame's
/// reflection reprojected, an eighth-size average taken, filtered across the surface and resolved over time into the
/// view's history, which combine reads. A ray the screen does not answer is listed for the scene pass
/// (`SceneTracePass`), which traces such rays alone against the level's static geometry.
pub struct ReflectionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  /// The list the trace leaves rays to the scene pass in.
  request_layout: wgpu::BindGroupLayout,
  pipelines: ReflectionPipelines,
  /// What a stage reads in place of a history or a stage before it where there is none: one texel of nothing.
  empty: wgpu::TextureView,
  /// What combine and the debug view read where nothing is traced: one texel marked untraced.
  untraced: wgpu::TextureView,
  /// What the ends view reads while nothing is traced: one texel of no ray.
  no_ends: wgpu::TextureView,
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
  upscale: wgpu::RenderPipeline,
  guide: wgpu::RenderPipeline,
}

impl ReflectionPass {
  /// The denoiser's stages, by their entry points.
  pub const REPROJECT: &'static str = "fs_reproject";
  pub const AVERAGE_STAGE: &'static str = "fs_average";
  pub const PREFILTER: &'static str = "fs_prefilter";
  pub const RESOLVE: &'static str = "fs_resolve";
  pub const UPSCALE: &'static str = "fs_upscale";
  pub const GUIDE: &'static str = "fs_guide";
  /// Each traced pixel's surface: its world normal packed, its roughness, its depth and its frame texel packed.
  pub const GUIDED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
  /// What the trace writes: what is reflected, then the ray's length in metres, below none where nothing is traced.
  pub const TRACED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// How each ray ended, a reason's number over 255 (`common/reflection_end.wgsl`), which the ends view colours.
  pub const ENDS: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
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
    queue: &wgpu::Queue,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = ReflectionParameters::create_layout(device);
    let request_layout: wgpu::BindGroupLayout = TraceRequestParameters::create_layout(device);
    let texel = |label: &str, format: wgpu::TextureFormat, bytes: &[u8]| -> wgpu::TextureView {
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
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
          },
          Default::default(),
          bytes,
        )
        .create_view(&Default::default())
    };

    Ok(Self {
      pipelines: Self::create_pipelines(source, (view_layout, &layout, sky_layout), &request_layout)?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      request_layout,
      // Half floats: nothing, and nothing with an alpha of minus one; then no ray.
      empty: texel("reflections empty", Self::TRACED, bytemuck::cast_slice(&[0u16; 4])),
      untraced: texel(
        "reflections none",
        Self::TRACED,
        bytemuck::cast_slice(&[0u16, 0, 0, 0xbc00]),
      ),
      no_ends: texel("reflection ends none", Self::ENDS, bytemuck::bytes_of(&0.0f32)),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("reflection history"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(
        source,
        (&self.view_layout, &self.layout, &self.sky_layout),
        &self.request_layout,
      ) {
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

  /// The texel the ends view reads while nothing is traced.
  pub fn get_no_ends(&self) -> &wgpu::TextureView {
    &self.no_ends
  }

  /// The history's bilinear sampler.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Traces every reflecting pixel's ray over the screen into the targets the pass draws into, listing in `requests`
  /// those it leaves to the scene pass.
  pub fn record_trace(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    (parameters, sky): (&ReflectionParameters<'_>, &SkyParameters<'_>),
    requests: &TraceRequestParameters,
  ) {
    context.bind(parameters);
    context.bind(sky);
    context.bind(requests);

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
      Self::UPSCALE => &self.pipelines.upscale,
      Self::GUIDE => &self.pipelines.guide,
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
    source: PipelineSource<'_>,
    (view_layout, layout, sky_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
    request_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<ReflectionPipelines> {
    let traced_layout: [Option<&wgpu::BindGroupLayout>; 4] =
      [Some(view_layout), Some(layout), Some(sky_layout), Some(request_layout)];
    let stage_layout: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let create = |layouts: &[Option<&wgpu::BindGroupLayout>],
                  fragment: &'static str,
                  formats: &[wgpu::TextureFormat]|
     -> XrfResult<wgpu::RenderPipeline> {
      let targets: Vec<Option<wgpu::ColorTargetState>> = formats.iter().map(|it| Some((*it).into())).collect();

      create_fullscreen_pipeline_into(source, "frame/reflections", fragment, layouts, &targets)
    };

    Ok(ReflectionPipelines {
      trace: create(&traced_layout, "fs_trace", &[Self::TRACED, Self::ENDS])?,
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
      upscale: create(&stage_layout, Self::UPSCALE, &[ReflectionHistory::RADIANCE_FORMAT])?,
      guide: create(&stage_layout, Self::GUIDE, &[Self::GUIDED])?,
    })
  }
}
