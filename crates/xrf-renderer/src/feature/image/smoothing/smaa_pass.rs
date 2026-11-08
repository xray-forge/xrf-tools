use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::image::smoothing::smaa_parameters::SmaaParameters;
use crate::feature::image::smoothing::smaa_stage::SmaaStage;
use crate::frame::targets::view_targets::ViewTargets;
use crate::host::render_bundle::RenderBundle;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;

/// `AreaTex` and `SearchTex` of SMAA v2.8 (MIT, Jorge Jimenez et al.), as three.js's `SMAANode` carries them, by their
/// paths in the renderer's bundle.
const AREA_TEXTURE: &str = "smaa/area.png";
const SEARCH_TEXTURE: &str = "smaa/search.png";

/// SMAA 1x over a viewport's scene as drawn: its edges, their blending weights, then the scene blended across them into
/// a target of its own, which the frame goes on from.
pub(crate) struct SmaaPass {
  layout: wgpu::BindGroupLayout,
  /// The edges', the weights' and the blend's.
  pipelines: PipelineSet<[wgpu::RenderPipeline; SmaaStage::COUNT]>,
  area: wgpu::TextureView,
  search: wgpu::TextureView,
  /// What a stage binds in place of the target it writes.
  empty: wgpu::TextureView,
  linear: wgpu::Sampler,
  point: wgpu::Sampler,
}

impl SmaaPass {
  /// The edges' and the blend weights' format.
  pub(crate) const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile, or a lookup texture cannot be read or decoded.
  pub(crate) fn new(queue: &wgpu::Queue, source: PipelineSource<'_>, bundle: &dyn RenderBundle) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = SmaaParameters::create_layout(device);
    let area: image::RgbImage =
      image::load_from_memory_with_format(&bundle.read_bundled(AREA_TEXTURE)?, image::ImageFormat::Png)
        .map_err(|error| XrfError::new_unexpected_error(format!("SMAA's area texture does not decode: {error}")))?
        .to_rgb8();
    let search: image::GrayImage =
      image::load_from_memory_with_format(&bundle.read_bundled(SEARCH_TEXTURE)?, image::ImageFormat::Png)
        .map_err(|error| XrfError::new_unexpected_error(format!("SMAA's search texture does not decode: {error}")))?
        .to_luma8();
    let area_texels: Vec<u8> = area.pixels().flat_map(|it| [it.0[0], it.0[1]]).collect();

    Ok(Self {
      pipelines: PipelineSet::new(source.cache, Self::create_pipelines(source, &layout)?),
      area: Self::create_lookup(
        device,
        queue,
        (area.width(), area.height()),
        wgpu::TextureFormat::Rg8Unorm,
        &area_texels,
      ),
      search: Self::create_lookup(
        device,
        queue,
        (search.width(), search.height()),
        wgpu::TextureFormat::R8Unorm,
        search.as_raw(),
      ),
      empty: Self::create_lookup(device, queue, (1, 1), wgpu::TextureFormat::Rgba8Unorm, &[0; 4]),
      linear: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("smaa linear"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      point: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("smaa point"),
        ..Default::default()
      }),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self
      .pipelines
      .refresh(source.cache, || Self::create_pipelines(source, &self.layout))
    {
      log::error!("SMAA rejected, smoothing with the last one: {error}");
    }
  }

  /// The area and search lookups, and the empty texture a stage reads in place of what it writes or precedes.
  pub(crate) fn get_lookups(&self) -> [&wgpu::TextureView; 3] {
    [&self.area, &self.search, &self.empty]
  }

  /// The linear and point samplers its stages read with.
  pub(crate) fn get_samplers(&self) -> [&wgpu::Sampler; 2] {
    [&self.linear, &self.point]
  }

  /// Draws one of its three stages into the target the pass draws into: the edges, the blend weights, then the blend.
  pub(crate) fn record(&self, context: &mut RasterContext<'_>, stage: SmaaStage, parameters: &SmaaParameters<'_>) {
    context.get_pass().set_pipeline(&self.pipelines[stage.get_index()]);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let create = |fragment: &'static str, format: wgpu::TextureFormat| {
      create_fullscreen_pipeline(source, "frame/smaa", fragment, &[Some(layout)], format)
    };

    Ok([
      create("fs_smaa_edges", Self::TARGET_FORMAT)?,
      create("fs_smaa_weights", Self::TARGET_FORMAT)?,
      create("fs_smaa_blend", ViewTargets::SCENE)?,
    ])
  }

  fn create_lookup(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    (width, height): (u32, u32),
    format: wgpu::TextureFormat,
    texels: &[u8],
  ) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;

    device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("smaa lookup"),
          size: wgpu::Extent3d {
            width,
            height,
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
        texels,
      )
      .create_view(&Default::default())
  }
}
