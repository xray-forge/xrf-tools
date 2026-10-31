use xrf_renderer_core::{GraphBuffer, GraphBufferAccess, GraphDeclaration, StorageValue, StorageValueMut};

use crate::frame::gpu_readback::GpuReadback;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sky_harmonics::SkyHarmonics;
use crate::pass::sky_irradiance_parameters::SkyIrradianceParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::scene::level::lighting_handles::LightingHandles;

/// The physical light's sky irradiance: the harmonics the GPU projects the sky into each frame, copied out without
/// waiting, and the last that came back, which the lighting uniform carries to every pass lighting by the hemisphere.
pub struct SkyIrradiance {
  buffer: wgpu::Buffer,
  readback: GpuReadback,
  harmonics: SkyHarmonics,
}

impl SkyIrradiance {
  pub fn new(device: &wgpu::Device) -> Self {
    let size: u64 = size_of::<SkyHarmonics>() as u64;

    Self {
      buffer: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sky harmonics"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      readback: GpuReadback::new(device, "sky harmonics readback", size),
      harmonics: SkyHarmonics::default(),
    }
  }

  /// The harmonics last read back, refreshed from a copy back since the last frame; none projected before the first.
  pub fn take_harmonics(&mut self) -> SkyHarmonics {
    if let Some(Ok(harmonics)) = self
      .readback
      .take(|bytes| bytemuck::pod_read_unaligned::<SkyHarmonics>(&bytes[..size_of::<SkyHarmonics>()]))
    {
      self.harmonics = harmonics;
    }

    self.harmonics
  }

  /// Projects the sky this frame and copies the harmonics out, unless an earlier copy is still on its way.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    (sky, lit): (&SkyParameters<'a>, &LightingHandles<'a>),
  ) {
    let buffer: GraphBuffer = bindings.import_buffer(&mut *graph, "sky harmonics", &self.buffer);
    let parameters: SkyIrradianceParameters = SkyIrradianceParameters {
      sky_cube_0: sky.sky_cube_0,
      sky_cube_1: sky.sky_cube_1,
      sky_environment_0: sky.sky_environment_0,
      sky_environment_1: sky.sky_environment_1,
      sky_clouds_0: sky.sky_clouds_0,
      sky_clouds_1: sky.sky_clouds_1,
      sky_clamp: sky.sky_clamp,
      sky_repeat: sky.sky_repeat,
      lighting: lit.lighting,
      exposure: StorageValue::new(lit.exposure),
      harmonics: StorageValueMut::new(buffer),
    };

    graph
      .add_compute_pass("sky irradiance")
      .parameters(&parameters)
      .record(move |context| passes.sky_irradiance.record(context, &parameters));
    graph
      .add_encoder_pass("sky irradiance readback")
      .buffer(buffer, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        if self.readback.is_free() {
          let source: &wgpu::Buffer = context.get_buffer(buffer);

          context
            .get_encoder()
            .copy_buffer_to_buffer(source, 0, self.readback.get_buffer(), 0, source.size());
          self.readback.mark_recorded();
        }
      });
  }

  /// Asks for the harmonics copied out with the frame just submitted.
  pub fn request(&self) {
    self.readback.request();
  }

  /// Drops the harmonics a frame that was never submitted copied out.
  pub fn abandon(&self) {
    self.readback.abandon();
  }

  /// Its buffer and readback.
  pub fn get_memory(&self) -> u64 {
    self.buffer.size() + self.readback.get_buffer().size()
  }
}
