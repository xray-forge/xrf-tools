use xrf_renderer_core::{FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, StorageValue, StorageValueMut};

use crate::contract::render_meter_reading::RenderMeterReading;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::gpu_readback::GpuReadback;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::meter_parameters::MeterParameters;
use crate::pass::meter_sample::MeterSample;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// A view's spot meter, while it meters a lit view: the sample its pass writes, the readback the sample is copied into
/// without waiting, and the last reading that came back.
#[derive(Default)]
pub struct MeterFeature {
  sample: Option<(wgpu::Buffer, GpuReadback)>,
  reading: Option<RenderMeterReading>,
}

impl MeterFeature {
  /// Makes its sample and readback while the view meters a lit scene; lets them and the last reading go otherwise.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions) {
    if !(options.mode.is_metered && options.mode.is_lit) {
      self.release();

      return;
    }

    if self.sample.is_none() {
      let size: u64 = size_of::<MeterSample>() as u64;

      self.sample = Some((
        device.create_buffer(&wgpu::BufferDescriptor {
          label: Some("meter sample"),
          size,
          usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
          mapped_at_creation: false,
        }),
        GpuReadback::new(device, "meter readback", size),
      ));
    }
  }

  /// Meters the scene finished so far at the view's centre and copies the sample out, unless an earlier copy is still on
  /// its way; nothing while it does not meter.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit): (ViewTargetHandles, LightingHandles<'a>),
  ) {
    let Some((buffer, readback)) = &self.sample else {
      return;
    };
    let sample: GraphBuffer = bindings.import_buffer(&mut *graph, "meter sample", buffer);
    let parameters: MeterParameters = MeterParameters {
      scene: targets.scene,
      albedo_target: targets.albedo,
      light_target: targets.light,
      depth_target: targets.depth,
      lighting: lit.lighting,
      exposure: StorageValue::new(lit.exposure),
      sample: StorageValueMut::new(sample),
    };

    graph
      .add_compute_pass("meter")
      .parameters(&parameters)
      .record(move |context| passes.meter.record(context, view, &parameters));
    graph
      .add_encoder_pass("meter readback")
      .buffer(sample, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        if readback.is_free() {
          let source: &wgpu::Buffer = context.get_buffer(sample);

          context
            .get_encoder()
            .copy_buffer_to_buffer(source, 0, readback.get_buffer(), 0, source.size());
          readback.mark_recorded();
        }
      });
  }

  /// Asks for the sample recorded with the frame just submitted.
  pub fn request(&self) {
    if let Some((_, readback)) = &self.sample {
      readback.request();
    }
  }

  /// Drops the sample a frame that was never submitted copied out.
  pub fn abandon(&self) {
    if let Some((_, readback)) = &self.sample {
      readback.abandon();
    }
  }

  /// The latest reading, refreshed once a sample is back; none while it does not meter or before its first.
  pub fn take_reading(&mut self) -> Option<RenderMeterReading> {
    let (_, readback) = self.sample.as_ref()?;

    if let Some(Ok(sample)) =
      readback.take(|bytes| bytemuck::pod_read_unaligned::<MeterSample>(&bytes[..size_of::<MeterSample>()]))
    {
      self.reading = Some(sample.to_reading());
    }

    self.reading
  }
}

impl ViewFeature for MeterFeature {
  fn get_memory(&self) -> u64 {
    self
      .sample
      .as_ref()
      .map_or(0, |(buffer, readback)| buffer.size() + readback.get_buffer().size())
  }

  fn release(&mut self) {
    self.sample = None;
    self.reading = None;
  }
}
