use crate::camera::camera_view::CameraView;
use crate::contract::render_light_shadow_filter::RenderLightShadowFilter;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::frame::stats_readback::StatsReadback;
use crate::pass::light_buffers::LightBuffers;
use crate::pass::light_record::LightRecord;
use crate::pass::lights_uniform::LightsUniform;

/// Lights standing in view at most in one frame: the nearest are kept.
pub const MAX_LIGHTS: usize = 1024;

/// Clusters the view is cut into, and lights one holds at most, as `shaders/common/light_clusters.wgsl` declares them.
const LIGHT_CLUSTERS: u64 = 16 * 9 * 24;
const LIGHT_CLUSTER_CAPACITY: u64 = 64;

/// The bytes after the clusters' counts the binning counts its overflow in, as a readback copies them.
const OVERFLOW_BYTES: u64 = 16;

/// A level's local lights as one view lights its frame with them: the nearest in its camera, written out by the scene's
/// `LevelLights::prepare` in its view space, the clusters the lights pass bins them into, and what they came to.
pub struct LightsView {
  pub records: Vec<LightRecord>,
  pub record_buffer: wgpu::Buffer,
  pub counts: wgpu::Buffer,
  pub items: wgpu::Buffer,
  pub uniform: wgpu::Buffer,
  count: u32,
  /// What the last frame's lights came to, and the binning's count of the clusters it filled, read back.
  pub report: RenderLightsReport,
  overflow: StatsReadback,
  /// What a zone's range strays by each frame.
  pub random: u64,
}

impl LightsView {
  pub fn new(device: &wgpu::Device) -> Self {
    let storage = |label: &str, size: u64| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    Self {
      records: Vec::with_capacity(MAX_LIGHTS),
      record_buffer: storage("light records", (MAX_LIGHTS * size_of::<LightRecord>()) as u64),
      // The clusters' counts, then the binning's two words of overflow.
      counts: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("light cluster counts"),
        size: LIGHT_CLUSTERS * 4 + OVERFLOW_BYTES,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      items: storage("light cluster items", LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY * 4),
      uniform: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lights"),
        size: size_of::<LightsUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      count: 0,
      report: RenderLightsReport::default(),
      overflow: StatsReadback::new(device),
      random: 0x9E37_79B9_7F4A_7C15,
    }
  }

  pub fn get_buffers(&self) -> LightBuffers<'_> {
    LightBuffers {
      records: &self.record_buffer,
      counts: &self.counts,
      items: &self.items,
      uniform: &self.uniform,
    }
  }

  pub fn get_count(&self) -> u32 {
    self.count
  }

  /// Writes the records filled this frame and what the passes bin and light them with.
  pub fn write(&mut self, queue: &wgpu::Queue, camera: &CameraView, filter: RenderLightShadowFilter) {
    let (near, far): (f32, f32) = camera.get_depth_range();

    self.count = self.records.len() as u32;
    self.report.in_view = self.count;
    queue.write_buffer(
      &self.uniform,
      0,
      bytemuck::bytes_of(&LightsUniform::new(self.count, camera.projection, near, far, filter)),
    );

    if !self.records.is_empty() {
      queue.write_buffer(&self.record_buffer, 0, bytemuck::cast_slice(&self.records));
    }
  }

  /// Clears the binning's overflow words before it counts this frame's.
  pub fn clear_overflow(&self, encoder: &mut wgpu::CommandEncoder) {
    encoder.clear_buffer(&self.counts, LIGHT_CLUSTERS * 4, Some(OVERFLOW_BYTES));
  }

  /// Copies the binning's overflow out with this frame's work, for a report a frame or more later.
  pub fn record_overflow(&self, encoder: &mut wgpu::CommandEncoder) {
    self.overflow.record(encoder, &self.counts, LIGHT_CLUSTERS * 4);
  }

  /// Asks for the overflow recorded with the frame just submitted.
  pub fn request_report(&self) {
    self.overflow.request();
  }

  /// What the last frame's lights came to, with the binning's overflow as last read back.
  pub fn take_report(&mut self) -> RenderLightsReport {
    let [full_clusters, dropped, ..] = self.overflow.take();

    RenderLightsReport {
      full_clusters,
      dropped,
      ..self.report
    }
  }
}
