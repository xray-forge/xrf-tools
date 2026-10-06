use glam::Mat4;

use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::pick_target::PickTarget;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::temporal_history::TemporalHistory;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::upscale_targets::UpscaleTargets;
use crate::frame::view_exposure::ViewExposure;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fsr_groups::FsrGroups;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_smoothing::LevelSmoothing;
use crate::scene::static_scene::static_selection::StaticSelection;

/// What one view keeps from one frame to the next: its targets and depth history, its temporal and upscaling state,
/// its exposure, what was asked of it (picks, the selection, overlays) and what it reads back.
pub struct ViewState {
  pub targets: Option<ViewTargets>,
  /// The depth pyramid over the targets, and its reduction's bind group a level.
  pub pyramid: Option<(DepthPyramid, Vec<wgpu::BindGroup>)>,
  /// Made again with the targets, which the cull's bind group follows.
  pub targets_epoch: u64,
  /// The view and projection the pyramid holds a frame's depth through, once one was reduced.
  pub history: Option<(Mat4, Mat4)>,
  /// The sequence each frame's samples are moved within their pixels by, while a temporal resolve gathers them.
  pub jitter: TemporalJitter,
  /// The temporal resolve's histories and its bind group writing each, while it resolves.
  pub temporal: Option<(TemporalHistory, [wgpu::BindGroup; 2])>,
  /// The last resolved frame's view projection without its jitter, and its view.
  pub temporal_previous: Option<(Mat4, Mat4)>,
  /// FSR 2's targets and bind groups, while it resolves.
  pub fsr: Option<(FsrTargets, FsrGroups)>,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  pub motion_previous: Option<Mat4>,
  /// The trees' sway the last frame drew with.
  pub last_wind: Option<WindUniform>,
  /// The frame at the viewport's size while it is drawn smaller, with its epoch, and the upscale passes' bind groups:
  /// EASU reading the scene, RCAS reading the upscaled frame.
  pub upscale: Option<(UpscaleTargets, [wgpu::BindGroup; 2])>,
  pub upscale_epoch: u64,
  /// The models' composited clusters this frame, back to front, as `(cluster, place)` entries, and the times it was
  /// made again, which its bind group follows.
  pub sorted_list: Option<wgpu::Buffer>,
  pub sorted_epoch: u64,
  /// What it draws over its frame.
  pub overlays: Option<LevelOverlays>,
  /// The selection box the overlays were made with, so a box that moved or came into the scene makes them again.
  pub overlays_box: Option<RenderOverlay>,
  /// What the selection marks, for the target it was resolved for and the scene generation it was resolved in.
  pub selection: Option<(RenderSelectionTarget, u64, Option<StaticSelection>)>,
  /// The smoothing pass while one smooths the scene as drawn.
  pub smoothing: Option<LevelSmoothing>,
  /// The eye's adaptation, carried from frame to frame.
  pub exposure: ViewExposure,
  /// What the cull kept, read back without waiting.
  pub stats: StatsReadback,
  /// The texel a pick is drawn into, and the camera narrowed to it.
  pub pick_target: Option<PickTarget>,
  pub pick_view: Option<ViewBinding>,
}

impl ViewState {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
      targets: None,
      pyramid: None,
      targets_epoch: 0,
      history: None,
      jitter: TemporalJitter::default(),
      temporal: None,
      temporal_previous: None,
      fsr: None,
      motion_previous: None,
      last_wind: None,
      upscale: None,
      upscale_epoch: 0,
      smoothing: None,
      overlays: None,
      overlays_box: None,
      selection: None,
      sorted_list: None,
      sorted_epoch: 0,
      exposure: ViewExposure::new(device, queue),
      stats: StatsReadback::new(device),
      pick_target: None,
      pick_view: None,
    }
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between; the
  /// same twice for a first frame.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    (current, self.motion_previous.replace(current).unwrap_or(current))
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }
}
