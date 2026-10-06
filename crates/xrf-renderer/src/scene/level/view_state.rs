use glam::Mat4;

use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::indirect_light_history::IndirectLightHistory;
use crate::frame::pick_target::PickTarget;
use crate::frame::reflection_history::ReflectionHistory;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::temporal_history::TemporalHistory;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::upscale_targets::UpscaleTargets;
use crate::frame::vbao_history::VbaoHistory;
use crate::frame::view_exposure::ViewExposure;
use crate::frame::view_targets::ViewTargets;
use crate::lighting::foliage_wind::FoliageWind;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::grass_view::GrassView;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_water::LevelWater;
use crate::scene::level::lights_view::LightsView;
use crate::scene::level::particles_view::ParticlesView;
use crate::scene::level::rain_wetness::RainWetness;
use crate::scene::static_scene::static_selection::StaticSelection;

/// What one view keeps from one frame to the next: its targets and depth history, its temporal and upscaling state,
/// its exposure, what was asked of it (picks, the selection, overlays) and what it reads back.
pub struct ViewState {
  pub targets: Option<ViewTargets>,
  /// The depth pyramid over the targets.
  pub pyramid: Option<DepthPyramid>,
  /// The view and projection the pyramid holds a frame's depth through, once one was reduced.
  pub history: Option<(Mat4, Mat4)>,
  /// The sequence each frame's samples are moved within their pixels by, while a temporal resolve gathers them.
  pub jitter: TemporalJitter,
  /// The temporal resolve's histories, while it resolves.
  pub temporal: Option<TemporalHistory>,
  /// The last resolved frame's view projection without its jitter, and its view.
  pub temporal_previous: Option<(Mat4, Mat4)>,
  /// FSR 2's targets, while it upscales.
  pub fsr: Option<FsrTargets>,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  pub motion_previous: Option<Mat4>,
  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  pub noise_frame: u32,
  /// The visibility-bitmask search's accumulations, while it accumulates.
  pub occlusion: Option<VbaoHistory>,
  /// The indirect light's accumulations, while it is gathered and accumulates.
  pub indirect: Option<IndirectLightHistory>,
  /// The reflections' blends over frames, while they are traced.
  pub reflections: Option<ReflectionHistory>,
  /// The water's settings and flow, and the enhanced water's reflection histories.
  pub water: LevelWater,
  /// The trees' sway the last frame drew with.
  pub last_wind: Option<WindUniform>,
  /// How far the enhanced foliage motion's flow fields have drifted.
  pub foliage: FoliageWind,
  /// How wet the level has become, built up by the rain and dried after.
  pub wetness: RainWetness,
  /// What the foliage motion read this frame, which the grass takes as the trees do.
  pub foliage_values: FoliageWindValues,
  /// The frame at the viewport's size while it is drawn smaller: EASU's upscale, then RCAS's sharpening.
  pub upscale: Option<UpscaleTargets>,
  /// The models' composited clusters this frame, back to front, as `(cluster, place)` entries.
  pub sorted_list: Option<wgpu::Buffer>,
  /// What it draws over its frame.
  pub overlays: Option<LevelOverlays>,
  /// The selection box the overlays were made with, so a box that moved or came into the scene makes them again.
  pub overlays_box: Option<RenderOverlay>,
  /// What the selection marks, for the target it was resolved for and the scene's contents it was resolved with.
  pub selection: Option<(RenderSelectionTarget, usize, Option<StaticSelection>)>,
  /// The eye's adaptation, carried from frame to frame.
  pub exposure: ViewExposure,
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
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
      targets: None,
      pyramid: None,
      history: None,
      jitter: TemporalJitter::default(),
      temporal: None,
      temporal_previous: None,
      fsr: None,
      motion_previous: None,
      noise_frame: 0,
      occlusion: None,
      indirect: None,
      reflections: None,
      water: LevelWater::default(),
      last_wind: None,
      foliage: FoliageWind::default(),
      wetness: RainWetness::default(),
      foliage_values: FoliageWindValues::default(),
      upscale: None,
      overlays: None,
      overlays_box: None,
      selection: None,
      sorted_list: None,
      exposure: ViewExposure::new(device, queue),
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

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }
}
