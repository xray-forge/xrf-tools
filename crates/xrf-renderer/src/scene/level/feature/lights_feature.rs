use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphRuntime, StorageArray,
  StorageArrayMut, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_light_shadow_filter::RenderLightShadowFilter;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::light_record::LightRecord;
use crate::pass::light_volume_record::LightVolumeRecord;
use crate::pass::light_volumes_uniform::LightVolumesUniform;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::light_buffers::LightBuffers;
use crate::scene::level::lighting_handles::LightingHandles;

/// Lights standing in view at most in one frame: the nearest are kept.
pub const MAX_LIGHTS: usize = 1024;

/// Clusters the view is cut into, and lights one holds at most, as `shaders/common/light_clusters.wgsl` declares them.
const LIGHT_CLUSTERS: u64 = 16 * 9 * 24;
const LIGHT_CLUSTER_CAPACITY: u64 = 64;

/// The bytes after the clusters' counts the binning counts its overflow in, as a readback copies them.
const OVERFLOW_BYTES: u64 = 16;

/// A level's local lights as one view lights its frame with them: the nearest in its camera, written out by the scene's
/// `LevelLights::prepare` in its view space, the clusters the lights pass bins them into, and what they came to.
pub struct LightsFeature {
  /// The lights as the view asks for them this frame.
  pub settings: RenderLightsSettings,
  pub records: Vec<LightRecord>,
  pub record_buffer: wgpu::Buffer,
  /// The shafts the shadowed lights draw this frame, the slices each draws, and what they all share.
  pub volumes: Vec<LightVolumeRecord>,
  pub volume_slices: Vec<u32>,
  pub volume_buffer: wgpu::Buffer,
  pub volumes_uniform: LightVolumesUniform,
  pub counts: wgpu::Buffer,
  pub items: wgpu::Buffer,
  /// What the passes bin and light the records with this frame.
  pub uniform: LightsUniform,
  count: u32,
  /// What the last frame's lights came to, and the binning's count of the clusters it filled, read back.
  pub report: RenderLightsReport,
  overflow: StatsReadback,
  /// What a zone's range strays by each frame.
  pub random: u64,
}

impl LightsFeature {
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
      settings: RenderLightsSettings::default(),
      records: Vec::with_capacity(MAX_LIGHTS),
      record_buffer: storage("light records", (MAX_LIGHTS * size_of::<LightRecord>()) as u64),
      volumes: Vec::with_capacity(MAX_LIGHTS),
      volume_slices: Vec::with_capacity(MAX_LIGHTS),
      volume_buffer: storage("light volumes", (MAX_LIGHTS * size_of::<LightVolumeRecord>()) as u64),
      volumes_uniform: LightVolumesUniform::default(),
      // The clusters' counts, then the binning's two words of overflow.
      counts: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("light cluster counts"),
        size: LIGHT_CLUSTERS * 4 + OVERFLOW_BYTES,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      items: storage("light cluster items", LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY * 4),
      uniform: LightsUniform::default(),
      count: 0,
      report: RenderLightsReport::default(),
      overflow: StatsReadback::new(device),
      random: 0x9E37_79B9_7F4A_7C15,
    }
  }

  pub fn get_count(&self) -> u32 {
    self.count
  }

  /// Writes the records filled this frame and what the passes bin and light them with.
  pub fn write(&mut self, encoder: &mut FrameEncoder, camera: &CameraView, filter: RenderLightShadowFilter) {
    let (near, far): (f32, f32) = camera.get_depth_range();

    self.count = self.records.len() as u32;
    self.report.in_view = self.count;
    self.uniform = LightsUniform::new(self.count, camera.projection, near, far, filter);

    if !self.records.is_empty() {
      encoder.write_buffer(&self.record_buffer, 0, bytemuck::cast_slice(&self.records));
    }

    self.report.volumetric = self.volumes.len() as u32;

    if !self.volumes.is_empty() {
      encoder.write_buffer(&self.volume_buffer, 0, bytemuck::cast_slice(&self.volumes));
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

  /// Drops the overflow recorded with a frame that was never submitted.
  pub fn abandon_report(&self) {
    self.overflow.abandon();
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

  /// Adds the clusters' counts cleared and, while any light stands in view, the records binned into the clusters;
  /// answers the lights as the graph holds them.
  pub fn add_binning_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
  ) -> LightBuffers {
    let buffers: LightBuffers = LightBuffers {
      counts: bindings.import_buffer(graph, "light cluster counts", &self.counts),
      records: bindings.import_buffer(graph, "light records", &self.record_buffer),
      items: bindings.import_buffer(graph, "light cluster items", &self.items),
      uniform: runtime.push_uniform(&self.uniform),
    };
    let counts: GraphBuffer = buffers.counts;

    graph
      .add_encoder_pass("light counts clear")
      .buffer(counts, GraphBufferAccess::CopyDestination)
      .record(move |context| self.clear_overflow(context.get_encoder()));

    if self.count > 0 {
      let parameters: LightBinningParameters = LightBinningParameters {
        records: StorageArray::new(buffers.records),
        counts: StorageArrayMut::new(buffers.counts),
        items: StorageArrayMut::new(buffers.items),
        lights: buffers.uniform,
      };

      graph
        .add_compute_pass("light binning")
        .parameters(&parameters)
        .record(move |context| passes.lights.record_binning(context, &parameters));
    }

    buffers
  }

  /// Adds every light in view drawn into the light target over the sun, through the scene's shadow `atlas`, each
  /// marching its contact shadows by `contact` where they are marched (`is_contact`); nothing while none stands in view.
  pub fn add_draw_passes<'a>(
    &self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (view, textures, atlas): (&'a ViewBinding, &'a wgpu::BindGroup, &'a wgpu::TextureView),
    (targets, lit, buffers): (ViewTargetHandles, LightingHandles<'a>, LightBuffers),
    (contact, is_contact): (UniformBinding<ContactShadowUniform>, bool),
  ) {
    if self.count == 0 {
      return;
    }

    let parameters: LightsParameters = LightsParameters {
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      records: StorageArray::new(buffers.records),
      counts: StorageArray::new(buffers.counts),
      items: StorageArray::new(buffers.items),
      lights: buffers.uniform,
      shadow_atlas: bindings.import_view(graph, "light shadow atlas", atlas),
      contact,
    };

    graph
      .add_raster_pass("lights")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(targets.light, wgpu::LoadOp::Load))
      .record(move |context| {
        passes
          .lights
          .record_draw(context, view, &parameters, (textures, is_contact))
      });
  }

  /// Adds the binning's overflow read back, for a report a frame or more later: an effect the graph cannot see.
  pub fn add_overflow_pass<'a>(&'a self, graph: &mut FrameGraph<'a>, buffers: LightBuffers) {
    graph
      .add_encoder_pass("light overflow")
      .buffer(buffers.counts, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| self.record_overflow(context.get_encoder()));
  }
}
