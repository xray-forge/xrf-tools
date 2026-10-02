use crate::camera::fly_camera_controller::FlyCameraController;
use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_camera::RenderCamera;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::grid_pass::GridPass;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

const SIZE: u32 = 64;

/// Draws the grid offscreen and reads it back: sky above the horizon, ground below. Skipped, and says so, on a
/// machine with no adapter at all; CI has WARP under D3D12.
#[test]
fn draws_sky_over_ground_offscreen() {
  let context: GpuContext = match GpuContext::create_headless(RenderBackend::D3d12)
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan))
  {
    Ok(context) => context,
    Err(error) => {
      eprintln!("Skipped: no GPU to draw with ({error})");

      return;
    }
  };
  let device: &wgpu::Device = &context.device;
  let format: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
  let grid: GridPass = GridPass::new(device, &shaders, &layout, format).unwrap();
  let binding: ViewBinding = ViewBinding::new(device, &layout);
  let camera: FlyCameraController = FlyCameraController::new(RenderCamera::Fly {
    position: [0.5, 2.0, 0.5],
    target: [0.5, 2.0, -10.0],
    field_of_view: 60.0,
    near: 0.1,
    far: 1000.0,
    speed: 1.0,
    boost: 1.0,
    sensitivity: 0.01,
  });

  binding.write(&context.queue, &CameraUniform::new(&camera.get_view(1.0), SIZE, SIZE));

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
    format,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: None,
    size: (SIZE * SIZE * 4) as u64,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let view: wgpu::TextureView = texture.create_view(&Default::default());
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

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

    grid.draw(&mut pass, &binding);
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
        bytes_per_row: Some(SIZE * 4),
        rows_per_image: Some(SIZE),
      },
    },
    wgpu::Extent3d {
      width: SIZE,
      height: SIZE,
      depth_or_array_layers: 1,
    },
  );
  context.queue.submit([encoder.finish()]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let pixels: Vec<u8> = readback.slice(..).get_mapped_range().unwrap().to_vec();
  let at = |x: u32, y: u32| -> [u8; 3] {
    let index: usize = ((y * SIZE + x) * 4) as usize;

    [pixels[index], pixels[index + 1], pixels[index + 2]]
  };
  let sky: [u8; 3] = at(SIZE / 2, 2);
  let ground: [u8; 3] = at(SIZE / 2, SIZE - 2);

  // Nothing left as cleared: the pass covers every pixel.
  assert_ne!(sky, [255, 0, 0]);
  assert_ne!(ground, [255, 0, 0]);
  // The sky is bluer than the ground under it.
  assert!(sky[2] > ground[2], "sky {sky:?} ground {ground:?}");
}
