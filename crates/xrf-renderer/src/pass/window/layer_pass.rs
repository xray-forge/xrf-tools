use xrf_error::XrfResult;
use xrf_renderer_core::{PipelineMap, RenderPipelineDescription, VertexLayout};

use crate::contract::layer::render_layer::RenderLayer;
use crate::contract::layer::render_layer_vertex::RenderLayerVertex;
use crate::frame::frame_encoder::FrameEncoder;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::viewport::layer::layer_draw::LayerDraw;
use crate::viewport::layer::layer_gpu::LayerGpu;
use crate::viewport::layer::layer_image::LayerImage;
use crate::viewport::layer::layer_region::LayerRegion;
use crate::viewport::layer::layer_texture::LayerTexture;
use crate::viewport::layer::viewport_layer::ViewportLayer;

/// Draws a window's layer (a native host's own interface) over everything else of the window, at the end of its
/// raster pass: clipped, textured, coloured triangles in device pixels, blended premultiplied as written, the window's
/// swapchain being display encoded. In the window's `Capture`, not in a viewport's `Screenshot`.
pub(crate) struct LayerPass {
  layout: wgpu::BindGroupLayout,
  linear: wgpu::Sampler,
  nearest: wgpu::Sampler,
  /// One a window format, built on first use.
  pipelines: PipelineMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
}

impl LayerPass {
  /// Bytes of the window size the vertex stage reads as immediates.
  const IMMEDIATE_SIZE: u32 = size_of::<[f32; 2]>() as u32;

  pub(crate) fn new(source: PipelineSource<'_>) -> Self {
    let device: &wgpu::Device = source.device;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("layer"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let sampler = |filter: wgpu::FilterMode| {
      device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("layer"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        ..Default::default()
      })
    };

    Self {
      linear: sampler(wgpu::FilterMode::Linear),
      nearest: sampler(wgpu::FilterMode::Nearest),
      layout,
      pipelines: PipelineMap::new(source.cache),
    }
  }

  /// Builds the pipelines again from a library reloaded since they were built; a broken edit keeps the last good ones.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipelines.refresh(source.cache, |format| {
      Self::create_pipeline(source, &self.layout, format)
    }) {
      log::error!("Layer shader rejected, drawing with the last one: {error}");
    }
  }

  /// Brings a viewport layer's GPU side up to what changed of it: its textures' changed regions written, a texture made
  /// again where its size or sampling changed, those freed let go, and the latest layer uploaded. Builds the pipeline
  /// drawing into `format` unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub(crate) fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut FrameEncoder,
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
    layer: &mut ViewportLayer,
  ) -> XrfResult {
    for (id, region) in std::mem::take(&mut layer.changed) {
      let Some(image) = layer.images.get(&id) else {
        layer.gpu.textures.remove(&id);
        continue;
      };
      let is_made: bool = layer.gpu.textures.get(&id).is_some_and(|texture| {
        texture.width == image.width && texture.height == image.height && texture.is_nearest == image.is_nearest
      });
      let region: LayerRegion = if is_made {
        region
      } else {
        layer.gpu.textures.insert(id, self.create_texture(device, image));

        LayerRegion::new(0, 0, image.width, image.height)
      };

      if let Some(texture) = layer.gpu.textures.get(&id) {
        Self::write_texture(queue, texture, image, region);
      }
    }

    if std::mem::take(&mut layer.is_layer_changed) {
      Self::upload(device, encoder, &mut layer.gpu, &layer.layer);
    }

    if !layer.is_drawn() {
      return Ok(());
    }

    self
      .pipelines
      .prepare(format, |format| Self::create_pipeline(source, &self.layout, format))
  }

  /// Draws a viewport's layer over a window `width` by `height` device pixels, each mesh within its clip and the window.
  pub(crate) fn draw(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    layer: &ViewportLayer,
  ) {
    let gpu: &LayerGpu = &layer.gpu;
    let (Some(pipeline), Some(vertices), Some(indices)) = (self.pipelines.get(&format), &gpu.vertices, &gpu.indices)
    else {
      return;
    };

    pass.set_pipeline(pipeline);
    pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
    pass.set_immediates(0, bytemuck::bytes_of(&[width as f32, height as f32]));
    pass.set_vertex_buffer(0, vertices.slice(..));
    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

    for draw in &gpu.draws {
      let (Some(texture), Some(clip)) = (gpu.textures.get(&draw.texture), draw.clip.clip(width, height)) else {
        continue;
      };

      pass.set_scissor_rect(clip.x.cast_unsigned(), clip.y.cast_unsigned(), clip.width, clip.height);
      pass.set_bind_group(0, &texture.bind_group, &[]);
      pass.draw_indexed(draw.indices.clone(), draw.base_vertex, 0..1);
    }
  }

  /// Writes `region` of `image` into its texture, read in place from the image's rows.
  fn write_texture(queue: &wgpu::Queue, texture: &LayerTexture, image: &LayerImage, region: LayerRegion) {
    if region.is_empty() {
      return;
    }

    queue.write_texture(
      wgpu::TexelCopyTextureInfo {
        texture: &texture.texture,
        mip_level: 0,
        origin: wgpu::Origin3d {
          x: region.left,
          y: region.top,
          z: 0,
        },
        aspect: wgpu::TextureAspect::All,
      },
      &image.pixels,
      wgpu::TexelCopyBufferLayout {
        offset: (u64::from(region.top) * u64::from(image.width) + u64::from(region.left)) * 4,
        bytes_per_row: Some(image.width * 4),
        rows_per_image: Some(region.get_height()),
      },
      wgpu::Extent3d {
        width: region.get_width(),
        height: region.get_height(),
        depth_or_array_layers: 1,
      },
    );
  }

  fn create_texture(&self, device: &wgpu::Device, image: &LayerImage) -> LayerTexture {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("layer"),
      size: wgpu::Extent3d {
        width: image.width,
        height: image.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      // Display encoded values sampled as written, as the swapchain shows them.
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let view: wgpu::TextureView = texture.create_view(&Default::default());
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("layer"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(if image.is_nearest { &self.nearest } else { &self.linear }),
        },
      ],
    });

    LayerTexture {
      texture,
      bind_group,
      width: image.width,
      height: image.height,
      is_nearest: image.is_nearest,
    }
  }

  /// Puts `sent`'s meshes in the layer's buffers, one run after another, growing a buffer that is too small.
  fn upload(device: &wgpu::Device, encoder: &mut FrameEncoder, layer: &mut LayerGpu, sent: &RenderLayer) {
    let mut vertices: Vec<RenderLayerVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    layer.draws.clear();

    for mesh in &sent.meshes {
      if mesh.indices.is_empty() || mesh.vertices.is_empty() {
        continue;
      }

      let (Ok(first), Ok(count), Ok(base_vertex)) = (
        u32::try_from(indices.len()),
        u32::try_from(mesh.indices.len()),
        i32::try_from(vertices.len()),
      ) else {
        break;
      };

      layer.draws.push(LayerDraw {
        clip: mesh.clip,
        texture: mesh.texture,
        indices: first..first + count,
        base_vertex,
      });
      vertices.extend_from_slice(&mesh.vertices);
      indices.extend_from_slice(&mesh.indices);
    }

    if layer.draws.is_empty() {
      return;
    }

    Self::write(
      device,
      encoder,
      &mut layer.vertices,
      wgpu::BufferUsages::VERTEX,
      bytemuck::cast_slice(&vertices),
    );
    Self::write(
      device,
      encoder,
      &mut layer.indices,
      wgpu::BufferUsages::INDEX,
      bytemuck::cast_slice(&indices),
    );
  }

  /// Writes `bytes` at the start of `buffer`, made anew at twice their size where it holds fewer.
  fn write(
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    buffer: &mut Option<wgpu::Buffer>,
    usage: wgpu::BufferUsages,
    bytes: &[u8],
  ) {
    let size: u64 = bytes.len() as u64;

    if buffer.as_ref().is_none_or(|buffer| buffer.size() < size) {
      *buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("layer"),
        size: (size * 2).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
    }

    if let Some(buffer) = buffer {
      encoder.write_buffer(buffer, 0, bytes);
    }
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
  ) -> XrfResult<wgpu::RenderPipeline> {
    source.get_render_pipeline(
      &RenderPipelineDescription::new("frame/layer", "frame/layer", "vs_layer")
        .with_fragment("fs_layer")
        .with_bind_group_layouts(&[Some(layout)])
        .with_immediate_size(Self::IMMEDIATE_SIZE)
        .with_vertex_layout(VertexLayout::new(
          size_of::<RenderLayerVertex>() as u64,
          wgpu::VertexStepMode::Vertex,
          &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
        ))
        .with_target(wgpu::ColorTargetState {
          format,
          blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        }),
    )
  }
}
