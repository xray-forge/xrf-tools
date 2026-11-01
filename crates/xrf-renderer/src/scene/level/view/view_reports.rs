use crate::contract::feature::image::resolve::render_antialiasing::RenderAntialiasing;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::report::scene::render_grass_report::RenderGrassReport;
use crate::contract::report::scene::render_lights_report::RenderLightsReport;
use crate::contract::report::scene::render_meter_reading::RenderMeterReading;
use crate::contract::report::scene::render_particles_report::RenderParticlesReport;
use crate::contract::report::scene::render_static_report::RenderStaticReport;

/// What a view reports of its recent frames, taken at once each report round: the counts its frames recorded once read
/// back, the sizes it draws and resolves at, how it smooths its frame, and what it draws with.
#[derive(Default)]
pub struct ViewReports {
  /// The static draws' pools and what the latest counted frame's cull kept and hid.
  pub static_draws: RenderStaticReport,
  pub lights: RenderLightsReport,
  /// The scene's particle simulation since the last report and the quads this view drew.
  pub particles: RenderParticlesReport,
  pub grass: RenderGrassReport,
  /// What the spot meter last read; none while the view does not meter.
  pub meter: Option<RenderMeterReading>,
  /// The size its scene is drawn at, none before its targets are made.
  pub render_size: Option<(u32, u32)>,
  /// The size its frame is resolved at, none before its targets are made.
  pub output_size: Option<(u32, u32)>,
  /// The temporal resolve, else the smoothing; none where neither runs.
  pub antialiasing: RenderAntialiasing,
  /// Whether DLSS reconstructs rays as it resolves the frame.
  pub is_ray_reconstructed: bool,
  /// What its frames are drawn with, as resolved from what its options asked; the weather's light is the viewport's to
  /// add.
  pub applied: RenderAppliedReport,
}
