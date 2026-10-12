use glam::Mat4;

use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::froxel_volume::FroxelVolume;
use crate::frame::pick_target::PickTarget;
use crate::frame::sky_prefilter::SkyPrefilter;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_targets::ViewTargets;
use crate::lighting::foliage_wind::FoliageWind;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::grass_view::GrassView;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::lights_view::LightsView;
use crate::scene::level::particles_view::ParticlesView;
use crate::scene::level::puddle_fill::PuddleFill;
use crate::scene::level::rain_wetness::RainWetness;
use crate::scene::static_scene::static_selection::StaticSelection;

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
  /// The sky as drawn, prefiltered by roughness into a cube's mips for the reflections' misses, while a sky is drawn.
  pub sky_prefilter: Option<SkyPrefilter>,
  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  pub noise_frame: u32,
  /// The enhanced volumetric lighting's froxels, while it draws.
  pub froxels: Option<FroxelVolume>,
  /// The trees' sway the last frame drew with.
  pub last_wind: Option<WindUniform>,
  /// How far the enhanced foliage motion's flow fields have drifted.
  pub foliage: FoliageWind,
  /// How wet the level has become, built up by the rain and dried after.
  pub wetness: RainWetness,
  pub puddle_fill: PuddleFill,
  /// What the foliage motion read this frame, which the grass takes as the trees do.
  pub foliage_values: FoliageWindValues,
  /// The models' composited clusters this frame, back to front, as `(cluster, place)` entries.
  pub sorted_list: Option<wgpu::Buffer>,
  /// What it draws over its frame.
  pub overlays: Option<LevelOverlays>,
  /// The selection box the overlays were made with, so a box that moved or came into the scene makes them again.
  pub overlays_box: Option<RenderOverlay>,
  /// What the selection marks, for the target it was resolved for and the scene's contents it was resolved with.
  pub selection: Option<(RenderSelectionTarget, usize, Option<StaticSelection>)>,
  /// What the cull kept, read back without waiting.
  pub stats: StatsReadback,
  /// The texel a pick is drawn into, and the camera narrowed to it.
  pub pick_target: Option<PickTarget>,
  pub pick_view: Option<ViewBinding>,
  /// The grass as it plants around its camera.
  pub grass: GrassView,
  /// The particles in its camera, as their quads.
  pub particles: ParticlesView,
  /// The lights in its camera, and their clusters.
  pub lights: LightsView,
}

impl ViewState {
  pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
    Self {
      targets: None,
      pyramid: None,
      history: None,
      motion_previous: None,
      view_projection_previous: None,
      sky_prefilter: None,
      noise_frame: 0,
      froxels: None,
      last_wind: None,
      foliage: FoliageWind::default(),
      wetness: RainWetness::default(),
      puddle_fill: PuddleFill::default(),
      foliage_values: FoliageWindValues::default(),
      overlays: None,
      overlays_box: None,
      selection: None,
      sorted_list: None,
      stats: StatsReadback::new(device),
      pick_target: None,
      pick_view: None,
      grass: GrassView::new(device),
      particles: ParticlesView::new(device),
      lights: LightsView::new(device),
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
