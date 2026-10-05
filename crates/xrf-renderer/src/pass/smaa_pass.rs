use xrf_error::{XrfError, XrfResult};

use crate::frame::smaa_targets::SmaaTargets;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_bundle::RenderBundle;
use crate::pass::fullscreen_pipeline::{begin_cleared_pass, create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::texture_entry;
use crate::shader::shader_library::ShaderLibrary;

/// `AreaTex` and `SearchTex` of SMAA v2.8 (MIT, Jorge Jimenez et al.), as three.js's `SMAANode` carries them, by their
/// paths in the renderer's bundle.
const AREA_TEXTURE: &str = "smaa/area.png";
const SEARCH_TEXTURE: &str = "smaa/search.png";

/// SMAA 1x over a viewport's scene as drawn: its edges, their blending weights, then the scene blended across them into
/// a target the scene is copied back from.
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
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile, or a lookup texture cannot be read or decoded.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let sampler = |binding: u32| wgpu::BindGroupLayoutEntry {
      binding,
      visibility: fragment,
      ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
      count: None,
    };
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("smaa"),
      entries: &[
        texture_entry(0, fragment, filtered, flat),
        texture_entry(1, fragment, filtered, flat),
        texture_entry(2, fragment, filtered, flat),
        texture_entry(3, fragment, filtered, flat),
        texture_entry(4, fragment, filtered, flat),
        sampler(5),
        sampler(6),
      ],
    });
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

  /// One bind group a stage: each binds an empty texture where it would read the target it writes.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    smaa: &SmaaTargets,
  ) -> [wgpu::BindGroup; 3] {
    [
      (&self.empty, &self.empty),
      (&smaa.edges, &self.empty),
      (&smaa.edges, &smaa.weights),
    ]
    .map(|(edges, weights)| {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("smaa"),
        layout: &self.layout,
        entries: &[
          texture_binding(0, &targets.scene),
          texture_binding(1, edges),
          texture_binding(2, weights),
          texture_binding(3, &self.area),
          texture_binding(4, &self.search),
          wgpu::BindGroupEntry {
            binding: 5,
            resource: wgpu::BindingResource::Sampler(&self.linear),
          },
          wgpu::BindGroupEntry {
            binding: 6,
            resource: wgpu::BindingResource::Sampler(&self.point),
          },
        ],
      })
    })
  }

  /// The three stages, the last blending into `target`.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    smaa: &SmaaTargets,
    groups: &[wgpu::BindGroup],
    target: &wgpu::TextureView,
  ) {
    for (index, (written, label)) in [
      (&smaa.edges, "smaa edges"),
      (&smaa.weights, "smaa weights"),
      (target, "smaa"),
    ]
    .into_iter()
    .enumerate()
    {
      let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass(encoder, label, written);

      pass.set_pipeline(&self.pipelines[index]);
      pass.set_bind_group(0, &groups[index], &[]);
      pass.draw(0..3, 0..1);
    }
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
      create("fs_smaa_edges", SmaaTargets::FORMAT)?,
      create("fs_smaa_weights", SmaaTargets::FORMAT)?,
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
