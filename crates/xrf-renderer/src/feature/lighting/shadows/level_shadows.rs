use glam::Vec3;

use crate::contract::feature::lighting::render_shadow_settings::{RENDER_MAX_SHADOW_CASCADES, RenderShadowSettings};
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::report::applied::render_applied_shadows::RenderAppliedShadows;
use crate::feature::lighting::shadows::cascades::shadow_cascade_view::ShadowCascadeView;
use crate::feature::lighting::shadows::cascades::sun_shadow_maps::SunShadowMaps;
use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;
use crate::feature::scene::drawable::casters::per_frame_casters::PerFrameCasters;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::lighting::sun::sun_cascade::SunCascade;

mod level_shadows_frame;
mod level_shadows_offers;
mod level_shadows_record;

/// Each cascade's passes, as the frame's timings name them, in the order a job lists them: a whole draw; its static
/// layer drawn, copied in and the trees drawn over it; the copy with the trees over it alone; its kept layer's copy;
/// and a band of its next static layer drawn ahead.
static WHOLE_PASSES: [[&str; 1]; RENDER_MAX_SHADOW_CASCADES] =
  [["sun shadow 0"], ["sun shadow 1"], ["sun shadow 2"], ["sun shadow 3"]];
static STATIC_PASSES: [[&str; 3]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow static 0", "sun shadow copy 0", "sun shadow sway 0"],
  ["sun shadow static 1", "sun shadow copy 1", "sun shadow sway 1"],
  ["sun shadow static 2", "sun shadow copy 2", "sun shadow sway 2"],
  ["sun shadow static 3", "sun shadow copy 3", "sun shadow sway 3"],
];
static SWAY_PASSES: [[&str; 2]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow sway 0", "sun shadow copy 0"],
  ["sun shadow sway 1", "sun shadow copy 1"],
  ["sun shadow sway 2", "sun shadow copy 2"],
  ["sun shadow sway 3", "sun shadow copy 3"],
];
static KEPT_PASSES: [[&str; 1]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow kept 0"],
  ["sun shadow kept 1"],
  ["sun shadow kept 2"],
  ["sun shadow kept 3"],
];
static BAND_PASSES: [[&str; 1]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow band 0"],
  ["sun shadow band 1"],
  ["sun shadow band 2"],
  ["sun shadow band 3"],
];

/// Milliseconds each cascade's whole draw costs before the timer measured one, the farther covering more; and its
/// trees over its static layer's copy.
const WHOLE_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.35, 0.45, 0.7, 1.0];
const SWAY_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.12, 0.15, 0.25, 0.35];
/// Milliseconds a kept layer's copy costs before the timer measured one.
const KEPT_SEED: f32 = 0.12;
/// Sway intervals a kept cascade's trees wait at most for another's redraw this frame.
const TREES_WAIT_MOST: f32 = 2.0;
/// Metres the camera moves and degrees it turns at most before trees kept only where it saw their shadows are drawn
/// again, within the widening of the frustum they were kept for.
const RECEIVED_MOVE_MOST: f32 = 4.0;
const RECEIVED_TURN_MOST: f32 = 10.0;

/// The share of a map's edge the sun's pass keeps a point off (`EDGE` in `common/sun_shadow.wgsl`).
const SUN_SHADOW_EDGE: f32 = 0.02;
/// Where the reach of what moves every frame is sampled for the cascades it lands in: rays a side of the screen's grid,
/// and the shares of the reach each ray is sampled at, close ones too, where the nearest cascade shades.
const SEEN_GRID: u32 = 5;
const SEEN_SHARES: [f32; 7] = [0.05, 0.1, 0.2, 0.35, 0.5, 0.75, 1.0];

/// The share of its map the nearest cascades blend into the next over at least, and how many of them do: their texels
/// are fine enough that a tuft's self-shadow differs from the next cascade's, which a band as narrow as the far ones'
/// shows as a pop where the cascade's edge sweeps across the grass with the camera.
const NEAR_BLEND: f32 = 0.2;
const NEAR_BLENDED: usize = 2;

/// The first cascade that keeps a second static layer, which its next is drawn into ahead of the turn that would stale
/// the one shown: the far ones, whose static draws cost the most.
const BANDED_FROM: usize = 2;
/// Bands a static layer drawn ahead, or a sweep of the trees' sway, is drawn in, at most one a frame (four frames of the
/// sway's 30 Hz at 120); and frames a band drawn ahead waits at most.
const BANDS: u32 = 4;
const BAND_WAIT_MOST: u32 = 2;
/// The share of what a static layer may hold past which its next is drawn ahead.
const PREFETCH_HOLD: f32 = 0.5;
/// Static layers kept: one a cascade, and a second for each drawn ahead.
const STATIC_LAYERS: u32 = (2 * RENDER_MAX_SHADOW_CASCADES - BANDED_FROM) as u32;

/// How a cascade due this frame is drawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CascadeDraw {
  /// Every caster into its map: while the trees stand still.
  Whole,
  /// What stands still into its static layer, copied into its map, the trees over it.
  Static,
  /// Its static layer copied into its map, the trees over it.
  Swaying,
  /// Its kept layer copied into its map, what moves every frame over it: the trees as last drawn, waiting their turn.
  Kept,
  /// One band of a cascade's rows: its static layer's rows restored into its map, the trees over them, so a sway
  /// redraw spreads over `BANDS` frames rather than landing on one.
  Banded(u32),
}

/// A cascade's draw offered this frame: which, its ticket, how it would be drawn, the fit it is drawn at, and what the
/// scene holds.
struct CascadeOffer {
  index: usize,
  ticket: PeriodicTicket,
  draw: CascadeDraw,
  fit: SunCascade,
  contents: usize,
}

/// The draw a cascade asks for: how it is drawn, the fit it is drawn at, and whether it is drawn at once whatever its
/// stagger, as one what moves every frame lands in is.
struct CascadeRequest {
  draw: CascadeDraw,
  fit: SunCascade,
  is_urgent: bool,
}

/// What a cascade keeping a static layer is offered by: whether what moves every frame lands in it, and whether another
/// kept cascade already redraws its trees this frame, which it notes where it redraws its own.
struct CascadeSplit<'s> {
  has_moving: bool,
  is_trees_taken: &'s mut bool,
}

/// A drawable kind's casters moving every frame, as this frame offered them, and the cascades they land in.
struct LandedCasters {
  casters: PerFrameCasters,
  cascades: [bool; RENDER_MAX_SHADOW_CASCADES],
}

/// A point of a kind's reach the camera sees: the cascade it is shaded by so far, as the sun's pass chooses one, and
/// whether it blends into the next there.
struct SeenPoint {
  point: Vec3,
  shaded_by: Option<usize>,
  is_blended: bool,
}

/// A level's sun shadow: its cascades fitted along the camera's view every frame, each culled and drawn into its map
/// only when its box moved, the scene changed under it, its trees' sway shows or what moves every frame lands in it.
/// Its casters go by how often they move: what stands still into a cascade's static layer, the trees swaying over its
/// copy at the sway's interval, and the drawable kinds' casters moving every frame (the grass's cards) over a kept layer
/// each frame.
#[derive(Default)]
pub struct LevelShadows {
  /// The maps, while any cascade is drawn.
  maps: Option<SunShadowMaps>,
  /// The cascades' static layers, while any cascade is drawn and the trees sway or anything moves every frame.
  statics: Option<SunShadowMaps>,
  /// The kept layers of the cascades what moves every frame reaches, while it casts: each its static layer with the
  /// trees over it as last drawn, which what moves is drawn over every frame while the trees redraw at their interval.
  kept: Option<SunShadowMaps>,
  values: ShadowUniform,
  cascades: Vec<ShadowCascadeView>,
  /// The cascades' draws offered this frame, and their bands drawn ahead, by cascade.
  offers: Vec<CascadeOffer>,
  band_offers: Vec<(usize, PeriodicTicket)>,
  /// The cascades this frame draws, as `commit_cascades` took them, and how; and the bands, by cascade, band and layer.
  due: Vec<(usize, CascadeDraw)>,
  bands: Vec<(usize, u32, u32)>,
  /// What moves every frame this frame, by kind, and the cascades it lands in: each the sun's pass samples somewhere in
  /// its reach the camera sees, so its shadows end where it ends (the grass's where the cards fade out) and not at the
  /// edge of a cascade the view turns.
  per_frame: Vec<LandedCasters>,
}

impl LevelShadows {
  /// Whether what moves every frame lands in a cascade this frame.
  fn has_moving(&self, index: usize) -> bool {
    self.per_frame.iter().any(|landed| landed.cascades[index])
  }

  /// How many of the nearest cascades casters standing `reach` metres from the eye are drawn into: the nearest whose
  /// width covers the reach and every nearer one.
  fn get_reached_count(settings: &RenderShadowSettings, reach: f32) -> usize {
    let count: usize = settings.get_cascade_count();

    (0..count)
      .position(|index| settings.get_cascade_width(index) >= reach)
      .map_or(count, |index| index + 1)
  }

  /// The maps, while any cascade is drawn.
  pub fn get_maps(&self) -> Option<&SunShadowMaps> {
    self.maps.as_ref()
  }

  /// Writes into `applied` the cascades `settings` ask for this frame and the maps' resolution, none without a cascade.
  pub fn describe(&self, applied: &mut RenderAppliedReport, settings: &RenderShadowSettings) {
    let cascades: usize = settings.get_cascade_count();

    applied.shadows = (cascades > 0).then(|| RenderAppliedShadows {
      cascades: settings.cascades[..cascades].to_vec(),
      resolution: self.maps.as_ref().map_or(0, |maps| maps.resolution),
      filter: settings.filter,
    });
  }

  /// What the passes lit by the sun sample its cascades by, as this frame fitted them.
  pub fn get_values(&self) -> &ShadowUniform {
    &self.values
  }

  /// Makes the maps at the `resolution` the settings ask for while any cascade is drawn, again where it changed, so the
  /// sun binds them before the frame draws; lets them go while none is (no `resolution`).
  pub fn prepare(&mut self, device: &wgpu::Device, resolution: Option<u32>) {
    let Some(resolution) = resolution else {
      self.release();

      return;
    };
    let resolution: u32 = resolution.clamp(1, device.limits().max_texture_dimension_2d);

    if self.maps.as_ref().is_none_or(|maps| maps.resolution != resolution) {
      self.maps = Some(SunShadowMaps::new(device, resolution));
      self.statics = None;
      self.kept = None;

      for (index, cascade) in self.cascades.iter_mut().enumerate() {
        cascade.drawn = None;
        cascade.forget_static(index);
      }
    }
  }

  /// Forgets that the cascades this frame drew were drawn, where its frame was not submitted: each draws again, and a
  /// static layer drawn ahead starts again.
  pub fn abandon_frame(&mut self) {
    for (index, _) in std::mem::take(&mut self.due) {
      self.cascades[index].drawn = None;
      self.cascades[index].drawn_static = None;
      self.cascades[index].is_kept = false;
      self.cascades[index].received_at = None;
      self.cascades[index].sway_sweep = None;
    }

    for (index, ..) in std::mem::take(&mut self.bands) {
      self.cascades[index].prefetch = None;
    }
  }
}

impl ViewFeature for LevelShadows {
  fn release(&mut self) {
    self.maps = None;
    self.statics = None;
    self.kept = None;

    for (index, cascade) in self.cascades.iter_mut().enumerate() {
      cascade.release(index);
    }
  }
}
