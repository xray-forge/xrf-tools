use glam::Mat4;

use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_targets::ViewTargets;
use crate::lighting::foliage_wind::FoliageWind;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::grass_view::GrassView;
use crate::scene::level::particles_view::ParticlesView;

/// What one view keeps from one frame to the next: its targets and depth history, what was asked of it (picks, the selection, overlays) and what it reads back.
pub struct ViewState {
  pub targets: Option<ViewTargets>,
  /// The depth pyramid over the targets.
  pub pyramid: Option<DepthPyramid>,
  /// The view and projection the pyramid holds a frame's depth through, once one was reduced.
  pub history: Option<(Mat4, Mat4)>,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  pub motion_previous: Option<Mat4>,
  /// The last frame's jittered view projection, which a read of its targets reprojects through.
  pub view_projection_previous: Option<Mat4>,
  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  pub noise_frame: u32,
  /// The trees' sway the last frame drew with.
  pub last_wind: Option<WindUniform>,
  /// How far the enhanced foliage motion's flow fields have drifted.
  pub foliage: FoliageWind,
  /// What the foliage motion read this frame, which the grass takes as the trees do.
  pub foliage_values: FoliageWindValues,
  /// What the cull kept, read back without waiting.
  pub stats: StatsReadback,
  /// The grass as it plants around its camera.
  pub grass: GrassView,
  /// The particles in its camera, as their quads.
  pub particles: ParticlesView,
}

impl ViewState {
  pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
    Self {
      targets: None,
      pyramid: None,
      history: None,
      motion_previous: None,
      view_projection_previous: None,
      noise_frame: 0,
      last_wind: None,
      foliage: FoliageWind::default(),
      foliage_values: FoliageWindValues::default(),
      stats: StatsReadback::new(device),
      grass: GrassView::new(device),
      particles: ParticlesView::new(device),
    }
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between; the
  /// same twice for a first frame.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    (current, self.motion_previous.replace(current).unwrap_or(current))
  }

  /// The last frame's jittered view projection, this frame's taking its place; this frame's own on the first.
  pub fn next_jittered(&mut self, current: Mat4) -> Mat4 {
    self.view_projection_previous.replace(current).unwrap_or(current)
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }
}
