use serde::{Deserialize, Serialize};

use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_dlss_availability::RenderDlssAvailability;
use crate::contract::render_frame_phases::RenderFramePhases;
use crate::contract::render_graph_report::RenderGraphReport;
use crate::contract::render_grass_report::RenderGrassReport;
use crate::contract::render_group_cost::RenderGroupCost;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_memory_report::RenderMemoryReport;
use crate::contract::render_meter_reading::RenderMeterReading;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pass_cost::RenderPassCost;
use crate::contract::render_static_report::RenderStaticReport;

/// What a viewport's recent frames cost, reported a few times a second while it draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFrameReport {
  /// Frames presented a second over the reported span.
  pub frames_per_second: f32,
  /// Mean milliseconds between presented frames.
  pub frame_time: f32,
  /// Longest milliseconds between two presented frames.
  pub frame_time_max: f32,
  /// Milliseconds between each two presented frames over the span, in order.
  pub frame_times: Vec<f32>,
  /// Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting.
  pub cpu_time: f32,
  /// Milliseconds of the render thread's own work each frame over the span, in order.
  pub cpu_times: Vec<f32>,
  /// Mean milliseconds the GPU spent on a frame from its first command to its last, whether or not its passes are
  /// timed; none where the device cannot time it.
  pub gpu_time: Option<f32>,
  /// Milliseconds the GPU spent on each frame read back over the span, in order; each comes back a few frames late.
  pub gpu_times: Vec<f32>,
  /// Where the render thread's time goes a frame, waiting for the window's image and presenting included.
  pub phases: RenderFramePhases,
  /// What each encode group cost the render thread a frame, in the order the frame records them.
  pub groups: Vec<RenderGroupCost>,
  /// Drawn width, in device pixels.
  pub width: u32,
  /// Drawn height, in device pixels.
  pub height: u32,
  /// The scene's width as rendered, smaller than the drawn one where it is upscaled.
  pub render_width: u32,
  /// And its height.
  pub render_height: u32,
  /// The width the frame is resolved at before it is rescaled to the drawn one.
  pub output_width: u32,
  /// And its height.
  pub output_height: u32,
  /// What smooths the frame, and upscales it where it resolves temporally.
  pub antialiasing: RenderAntialiasing,
  /// Whether DLSS reconstructs rays as it upscales the frame.
  pub is_ray_reconstructed: bool,
  /// The graphics API drawn with.
  pub backend: RenderBackend,
  /// The GPU drawn on.
  pub adapter: String,
  /// Whether it traces rays, by hardware ray query: what the reflections' and the water's tracing of what the screen
  /// misses needs.
  pub has_ray_queries: bool,
  /// Whether it upscales with DLSS, and what it lacks where it does not.
  pub dlss: RenderDlssAvailability,
  /// Whether it reconstructs rays with DLSS, and what it lacks where it does not.
  pub ray_reconstruction: RenderDlssAvailability,
  /// Whether its passes were timed on the GPU over the span.
  pub is_gpu_timed: bool,
  /// What each pass cost on the GPU, in frame order; none while untimed.
  pub passes: Vec<RenderPassCost>,
  /// The level's static draws' pools and cull, empty without a level.
  pub static_draws: RenderStaticReport,
  /// The level's local lights, empty without a level.
  pub lights: RenderLightsReport,
  /// The level's particle systems, empty without a level.
  pub particles: RenderParticlesReport,
  /// The view's grass, empty without a level or grass.
  pub grass: RenderGrassReport,
  /// Milliseconds the last sector taken in took to put into the scene, on the render thread.
  pub sector_time: f32,
  /// What the renderer holds on the GPU.
  pub memory: RenderMemoryReport,
  /// What the view's spot meter read, a frame or more late; none while it does not meter.
  pub meter: Option<RenderMeterReading>,
  /// What the frame graph made of its latest frame, none before its first.
  pub graph: Option<Box<RenderGraphReport>>,
}
