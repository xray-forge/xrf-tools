use std::sync::Arc;

use wgpu::util::StagingBelt;

use crate::contract::layer::render_layer::RenderLayer;
use crate::contract::layer::render_layer_mesh::RenderLayerMesh;
use crate::contract::layer::render_layer_texture::RenderLayerTexture;
use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;
use crate::contract::layer::render_layer_textures::RenderLayerTextures;
use crate::contract::layer::render_layer_vertex::RenderLayerVertex;
use crate::contract::viewport::render_rect::RenderRect;
use crate::frame::frame_encoder::FrameEncoder;
use crate::host::render_bundle::RenderBundle;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::support::test_bundle::TestBundle;
use crate::tests::support::test_context::create_test_context;
use crate::tests::support::test_workers::create_workers;
use crate::thread::gpu::running_gpu::RunningGpu;
use crate::thread::gpu::starting_gpu::StartingGpu;
use crate::viewport::layer::viewport_layer::ViewportLayer;

const SIZE: u32 = 16;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const WHITE: RenderLayerTextureId = RenderLayerTextureId(1);

/// A layer whose one white texture covers the window's left half.
fn create_layer() -> ViewportLayer {
  let mut layer: ViewportLayer = ViewportLayer::default();
  let half: f32 = (SIZE / 2) as f32;
  let corner = |x: f32, y: f32| RenderLayerVertex {
    position: [x, y],
    uv: [0.0, 0.0],
    color: [255; 4],
  };

  layer.change_textures(RenderLayerTextures {
    set: vec![RenderLayerTexture {
      id: WHITE,
      origin: None,
      width: 1,
      height: 1,
      pixels: vec![255; 4],
      is_nearest: true,
    }],
    free: vec![],
  });
  layer.set(RenderLayer {
    meshes: vec![RenderLayerMesh {
      clip: RenderRect {
        x: 0,
        y: 0,
        width: SIZE,
        height: SIZE,
      },
      texture: WHITE,
      vertices: vec![
        corner(0.0, 0.0),
        corner(half, 0.0),
        corner(half, SIZE as f32),
        corner(0.0, SIZE as f32),
      ],
      indices: vec![0, 1, 2, 0, 2, 3],
    }],
  });

  layer
}

/// Draws `layer` with the running GPU's layer pass into a red target and reads back its left and right halves.
fn draw_layer(running: &RunningGpu, layer: &ViewportLayer) -> [[u8; 4]; 2] {
  let device: &wgpu::Device = &running.gpu.context.device;
  let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: SIZE,
      height: SIZE,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  // Rows padded to the copy's alignment.
  let row: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: None,
    size: u64::from(row * SIZE),
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let view: wgpu::TextureView = texture.create_view(&Default::default());
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

  {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::RED),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });

    running.surfaces.layer.draw(&mut pass, FORMAT, SIZE, SIZE, layer);
  }

  encoder.copy_texture_to_buffer(
    wgpu::TexelCopyTextureInfo {
      texture: &texture,
      mip_level: 0,
      origin: wgpu::Origin3d::ZERO,
      aspect: wgpu::TextureAspect::All,
    },
    wgpu::TexelCopyBufferInfo {
      buffer: &readback,
      layout: wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(row),
        rows_per_image: Some(SIZE),
      },
    },
    wgpu::Extent3d {
      width: SIZE,
      height: SIZE,
      depth_or_array_layers: 1,
    },
  );

  let (commands, _): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

  running.gpu.context.queue.submit([commands]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let pixels: Vec<u8> = readback.slice(..).get_mapped_range().unwrap().to_vec();
  let at = |x: u32| -> [u8; 4] {
    let index: usize = ((SIZE / 2) * row + x * 4) as usize;

    [pixels[index], pixels[index + 1], pixels[index + 2], pixels[index + 3]]
  };

  [at(SIZE / 4), at(SIZE * 3 / 4)]
}

/// A layer brought up while the passes are built is drawn by the running GPU they make, its textures and buffers
/// carried on as they were uploaded rather than made again.
#[test]
fn hands_a_layer_brought_up_while_starting_on_to_the_running_gpu() {
  let Some(context) = create_test_context("to draw a layer with") else {
    return;
  };
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let bundle: Arc<dyn RenderBundle> = Arc::new(TestBundle::checkout());
  let mut starting: StartingGpu = StartingGpu::begin(context, &shaders, &create_workers(), &bundle, None).unwrap();
  let mut layer: ViewportLayer = create_layer();
  let mut encoder: FrameEncoder = starting.begin_encoder();

  starting
    .surfaces
    .layer
    .prepare(
      &starting.context.device,
      &starting.context.queue,
      &mut encoder,
      PipelineSource {
        cache: &starting.pipelines,
        device: &starting.context.device,
        shaders: &shaders,
      },
      FORMAT,
      &mut layer,
    )
    .unwrap();

  let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

  starting.context.queue.submit([commands]);
  starting.belt = Some(belt);

  assert!(layer.is_drawn(), "the starting GPU uploaded the layer");
  assert!(layer.changed.is_empty() && !layer.is_layer_changed);

  let running: RunningGpu = match starting.finish() {
    Ok(running) => running,
    Err(error) => panic!("The passes cannot be built: {error}"),
  };

  assert!(running.belt.is_some(), "the frames' belt goes on into the running GPU");
  assert_eq!(draw_layer(&running, &layer), [[255; 4], [255, 0, 0, 255]]);
}
