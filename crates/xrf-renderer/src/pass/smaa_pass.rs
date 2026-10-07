use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::host::render_bundle::RenderBundle;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::smaa_parameters::SmaaParameters;
use crate::shader::shader_library::ShaderLibrary;

/// `AreaTex` and `SearchTex` of SMAA v2.8 (MIT, Jorge Jimenez et al.), as three.js's `SMAANode` carries them, by their
/// paths in the renderer's bundle.
const AREA_TEXTURE: &str = "smaa/area.png";
const SEARCH_TEXTURE: &str = "smaa/search.png";

/// SMAA 1x over a viewport's scene as drawn: its edges, their blending weights, then the scene blended across them into
/// a frame's target the scene is copied back from.
pub struct SmaaPass {
  layout: wgpu::BindGroupLayout,
  /// The edges', the weights' and the blend's.
  pipelines: [wgpu::RenderPipeline; 3],
  area: wgpu::TextureView,
  search: wgpu::TextureView,
  /// What a stage binds in place of the target it writes.
  empty: wgpu::TextureView,
  linear: wgpu::Sampler,
  point: wgpu::Sampler,
  generation: u64,
}

impl SmaaPass {
  /// The edges' and the blend weights' format.
  pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile, or a lookup texture cannot be read or decoded.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
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
      pipelines: Self::create_pipelines(device, shaders, &layout)?,
      area: create_lookup(
        device,
        queue,
        (area.width(), area.height()),
        wgpu::TextureFormat::Rg8Unorm,
        &area_texels,
      ),
      search: create_lookup(
        device,
        queue,
        (search.width(), search.height()),
        wgpu::TextureFormat::R8Unorm,
        search.as_raw(),
      ),
      empty: create_lookup(device, queue, (1, 1), wgpu::TextureFormat::Rgba8Unorm, &[0; 4]),
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
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("SMAA rejected, smoothing with the last one: {error}"),
      }
    }
  }

  /// The area and search lookups, and the empty texture a stage reads in place of what it writes or precedes.
  pub fn get_lookups(&self) -> [&wgpu::TextureView; 3] {
    [&self.area, &self.search, &self.empty]
  }

  /// The linear and point samplers its stages read with.
  pub fn get_samplers(&self) -> [&wgpu::Sampler; 2] {
    [&self.linear, &self.point]
  }

  /// Draws one of its three stages into the target the pass draws into: the edges, the blend weights, then the blend.
  pub fn record(&self, context: &mut RasterContext<'_>, stage: usize, parameters: &SmaaParameters<'_>) {
    context.get_pass().set_pipeline(&self.pipelines[stage]);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let create = |fragment: &str, format: wgpu::TextureFormat| {
      create_fullscreen_pipeline(device, shaders, "frame/smaa", fragment, &[Some(layout)], format)
    };

    Ok([
      create("fs_smaa_edges", Self::TARGET_FORMAT)?,
      create("fs_smaa_weights", Self::TARGET_FORMAT)?,
      create("fs_smaa_blend", ViewTargets::SCENE)?,
    ])
  }
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
