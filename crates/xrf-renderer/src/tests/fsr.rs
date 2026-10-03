use glam::{Mat4, Vec2, Vec3};

use crate::camera::camera_view::CameraView;
use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fsr_groups::FsrGroups;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::fsr_uniform::FsrUniform;
use crate::shader::shader_library::ShaderLibrary;

/// Builds FSR 2 for a drawing upscaled half again, binds it to real targets and runs two frames, the second reading
/// the first's history, without a validation error. Skipped, and says so, on a machine with no adapter at all.
#[test]
fn upscales_two_frames_without_a_validation_error() {
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
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let pass: FsrPass = FsrPass::new(device, &shaders).unwrap();
  let (render, display): ((u32, u32), (u32, u32)) = ((64, 48), (96, 72));
  let targets: ViewTargets = ViewTargets::new(device, render.0, render.1);
  let mut fsr: FsrTargets = FsrTargets::new(device, render, display, ViewTargets::SCENE);
  let uniform: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: None,
    size: size_of::<FsrUniform>() as u64,
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });
  let groups: FsrGroups = pass.create_bind_groups(device, &targets, &fsr, &uniform);
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 4.0 / 3.0, 0.2, 1000.0);
  let mut jitter: TemporalJitter = TemporalJitter::default();

  for _ in 0..2 {
    let offset: Vec2 = jitter.next(1.5);
    let constants: FsrUniform = FsrUniform::new(
      (render, display),
      offset,
      &view,
      (FsrTargets::get_luma_mip_size(render), TemporalJitter::get_phases(1.5)),
      fsr.frame_index,
    );
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

    context.queue.write_buffer(&uniform, 0, bytemuck::bytes_of(&constants));
    pass.draw(&mut encoder, &fsr, &groups, &mut |_, _| {});
    context.queue.submit([encoder.finish()]);
    fsr.swap();
  }

  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
  assert_eq!(fsr.frame_index, 2);
  assert_eq!(FsrTargets::get_luma_mip_size(render), (2, 1));
}

/// The constants FSR 2 reads, for a lens with reversed depth: device depth to view depth by `y / (d - x)`.
#[test]
fn reads_view_depth_back_from_reversed_device_depth() {
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 1.0, 0.5, 100.0);
  let constants: FsrUniform = FsrUniform::new(((50, 50), (100, 100)), Vec2::new(0.25, -0.25), &view, ((1, 1), 32), 3);
  let to_view = |depth: f32| constants.device_to_view[1] / (depth - constants.device_to_view[0]);

  assert!((to_view(1.0) - 0.5).abs() < 1e-3);
  assert!((to_view(0.0) - 100.0).abs() < 1e-2);
  assert_eq!(constants.jitter, [-0.25, 0.25]);
  assert_eq!(constants.downscale, [0.5, 0.5]);
  assert_eq!(constants.frame_index, 3.0);
}
