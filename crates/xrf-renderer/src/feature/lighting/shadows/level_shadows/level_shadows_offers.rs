use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_frustum::CameraFrustum;
use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::feature::lighting::shadows::cascades::shadow_cascade_view::ShadowCascadeView;
use crate::feature::lighting::shadows::cascades::shadow_prefetch::ShadowPrefetch;
use crate::feature::lighting::shadows::level_shadows::{
  BAND_PASSES, BAND_WAIT_MOST, BANDED_FROM, BANDS, CascadeDraw, CascadeOffer, CascadeRequest, CascadeSplit,
  KEPT_PASSES, KEPT_SEED, LevelShadows, NEAR_BLEND, NEAR_BLENDED, PREFETCH_HOLD, RECEIVED_MOVE_MOST,
  RECEIVED_TURN_MOST, SEEN_GRID, SEEN_SHARES, STATIC_PASSES, SUN_SHADOW_EDGE, SWAY_PASSES, SWAY_SEEDS, SeenPoint,
  TREES_WAIT_MOST, WHOLE_PASSES, WHOLE_SEEDS,
};
use crate::feature::lighting::shadows::shadow_cut::ShadowCut;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::lighting::sun::sun_cascade::SunCascade;
use crate::lighting::sun::sun_view_rays::SunViewRays;
use crate::scene::level::shadow::shadow_frame::ShadowFrame;

impl LevelShadows {
  pub(super) fn offer_cascades(
    &mut self,
    device: &wgpu::Device,
    view_layout: &wgpu::BindGroupLayout,
    frame: &ShadowFrame<'_>,
    count: usize,
    scheduler: &mut PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let resolution: u32 = self.maps.as_ref().map_or(1, |maps| maps.resolution);
    let is_split: bool = self.statics.is_some();
    let blends: Vec4 = self.values.blends;
    let mut rays: SunViewRays = SunViewRays::new(frame.camera);
    // How far along the light the level stands, which every cascade's casters and receivers stand within.
    let span: Option<(f32, f32)> = scene.get_span(frame.sun_direction.normalize_or_zero());

    while self.cascades.len() < count.min(RENDER_MAX_SHADOW_CASCADES) {
      let index: usize = self.cascades.len();

      self.cascades.push(ShadowCascadeView::new(device, view_layout, index));
    }

    // The points of each kind's reach the camera sees, as the sun's pass shades them (`sun_shadow`): the cascades tile
    // the view, the far ones starting where the near ones end, so each shades a part of it.
    let mut sampled: Vec<Vec<SeenPoint>> = self
      .per_frame
      .iter()
      .map(|landed| {
        Self::sample_seen(frame.camera, landed.casters.reach + landed.casters.standing)
          .into_iter()
          .map(|point| SeenPoint {
            point,
            shaded_by: None,
            is_blended: false,
          })
          .collect()
      })
      .collect();
    // Whether a kept cascade already redraws its trees this frame: the others wait theirs, so the redraws spread.
    let mut is_trees_taken: bool = false;

    for (index, cascade) in self.cascades.iter_mut().take(count).enumerate() {
      let width: f32 = frame.settings.get_cascade_width(index);

      cascade
        .cascade
        .fit(frame.camera, &mut rays, frame.sun_direction, width, resolution, span);

      if !is_split {
        self.offers.extend(Self::offer_whole(index, cascade, frame, scheduler));

        continue;
      }

      let projection: Mat4 = cascade.cascade.view.get_view_projection();
      let has_next: bool = index + 1 < count;

      // A kind lands in a cascade where the cascade first shades a point of its reach, or blends into it from the one
      // nearer, while its reach draws into the cascade at all.
      for (landed, points) in self.per_frame.iter_mut().zip(&mut sampled) {
        let is_reached: bool = index < Self::get_reached_count(frame.settings, landed.casters.reach);

        for seen in points.iter_mut() {
          match (seen.shaded_by, Self::get_cascade_inset(projection, seen.point)) {
            (None, Some(inset)) => {
              landed.cascades[index] = is_reached;
              seen.shaded_by = Some(index);
              seen.is_blended = has_next && inset < blends[index];
            }
            (Some(at), Some(_)) if seen.is_blended && at + 1 == index => landed.cascades[index] = is_reached,
            _ => {}
          }
        }
      }

      let has_moving: bool = self.per_frame.iter().any(|landed| landed.cascades[index]);

      Self::swap_prefetched(index, cascade);
      self.offers.extend(Self::offer_split(
        index,
        cascade,
        frame,
        CascadeSplit {
          has_moving,
          is_trees_taken: &mut is_trees_taken,
        },
        scheduler,
      ));

      // What moves every frame is drawn with the cascade wherever it lands, which then always draws this frame: offered
      // for what it costs, a job a kind.
      for landed in self.per_frame.iter().filter(|landed| landed.cascades[index]) {
        scheduler.offer(PeriodicJob::new_now(
          std::slice::from_ref(&landed.casters.passes[index]),
          landed.casters.seed,
          1.0,
        ));
      }

      if index >= BANDED_FROM {
        cascade
          .band
          .get_or_insert_with(|| ShadowCut::new(device, view_layout, "sun shadow band"));

        let ticket: Option<PeriodicTicket> = Self::offer_band(index, cascade, frame, scheduler);

        self.band_offers.extend(ticket.map(|ticket| (index, ticket)));
      }
    }
  }

  /// Offers the draw a cascade drawn whole wants this frame, while the trees stand still: where its box moved, the
  /// scene changed under it or the sway shows.
  fn offer_whole(
    index: usize,
    cascade: &mut ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
  ) -> Option<CascadeOffer> {
    let scene = frame.scene;
    let fit: SunCascade = cascade.cascade;
    // A map holds depth in the world, sampled with the matrix it was drawn with, so a still one is still exact; only an
    // item coming or going within what it covers changes it.
    let is_changed: bool = cascade.has_drawn_moving
      || cascade.drawn.is_none_or(|(version, contents)| {
        let frustum: CameraFrustum = fit.view.get_frustum();

        version != fit.version || scene.is_changed_since(contents, |sphere| frustum.reaches(sphere))
      });

    let interval: f32 = frame.settings.get_sway_interval();

    if !(is_changed
      || frame.settings.is_swaying(index)
        && frame
          .sway
          .is_redrawn_every(frame.sway.reach, fit.texel, cascade.drawn_at, interval))
    {
      cascade.waited = 0;

      return None;
    }

    Some(Self::offer(
      index,
      cascade,
      frame,
      scheduler,
      CascadeRequest {
        draw: CascadeDraw::Whole,
        fit,
        is_urgent: cascade.has_drawn_moving,
      },
    ))
  }

  /// Offers the draw a cascade keeping a static layer wants this frame, while the trees sway or anything moves every
  /// frame: over its static layer, at the fit that layer was drawn at, while the cascade's fit holds by it and nothing
  /// changed under it (the nearest cascade's only while its fit is the same); else a new static layer at the cascade's
  /// fit. One what moves every frame lands in (`has_moving`) draws every frame, at once: the grass sways every frame,
  /// and a shadow drawn at the sway's interval would trail the blades drawn over it wherever the frame rate passes it.
  /// Its trees still wait their interval: between their draws it copies in its kept layer and draws what moves alone
  /// over it, and while another kept cascade redraws its trees this frame (`is_trees_taken`) one whose trees are due for
  /// the sway alone waits a frame, up to `TREES_WAIT_MOST` intervals, so the redraws spread over frames rather than
  /// piling into one.
  fn offer_split(
    index: usize,
    cascade: &mut ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    CascadeSplit {
      has_moving,
      is_trees_taken,
    }: CascadeSplit<'_>,
    scheduler: &mut PeriodicScheduler,
  ) -> Option<CascadeOffer> {
    let scene = frame.scene;
    let current: SunCascade = cascade.cascade;
    let static_fit: SunCascade = cascade.static_fit;
    let is_held: bool = cascade.drawn_static.is_some_and(|(version, contents)| {
      let is_fit: bool = if index == 0 {
        version == current.version
      } else {
        current.is_held_by(&static_fit)
      };

      let frustum: CameraFrustum = static_fit.view.get_frustum();

      is_fit && !scene.is_changed_since(contents, |sphere| frustum.reaches(sphere))
    });
    // Trees kept only where the camera saw their shadows are drawn again once it turned or moved far from where it was.
    let is_left: bool = cascade.received_at.is_some_and(|(eye, look)| {
      eye.distance(frame.camera.position) > RECEIVED_MOVE_MOST
        || look.dot(frame.camera.get_look()) < RECEIVED_TURN_MOST.to_radians().cos()
    });
    let is_stale: bool = cascade.drawn.is_none() || !is_held || cascade.is_swapped || is_left;
    let interval: f32 = frame.settings.get_sway_interval();
    let is_swayed: bool = frame.settings.is_swaying(index)
      && frame
        .sway
        .is_redrawn_every(frame.sway.reach, current.texel, cascade.drawn_at, interval);
    let is_kept: bool = has_moving && cascade.is_kept;
    let can_wait: bool = frame.sway.can_wait(cascade.drawn_at, interval * TREES_WAIT_MOST);
    let is_trees_due: bool = is_stale || (is_swayed && !(is_kept && *is_trees_taken && can_wait));

    if is_kept && is_trees_due {
      *is_trees_taken = true;
    }

    // A cascade redraws its trees for the sway alone band by band, a sweep at a time, unless what moves every frame lands
    // in it, which draws over its kept layer every frame. A near cascade's bands are drawn on the frame they are due,
    // so its sweep keeps the sway's rate and its rows a frame apart rather than one frame carrying the whole redraw.
    if !has_moving && !is_stale && (cascade.sway_sweep.is_some() || is_swayed) {
      let band: u32 = cascade.sway_sweep.map_or(0, |(_, band)| band);

      return Some(Self::offer(
        index,
        cascade,
        frame,
        scheduler,
        CascadeRequest {
          draw: CascadeDraw::Banded(band),
          fit: static_fit,
          is_urgent: index < BANDED_FROM,
        },
      ));
    }

    if is_kept && !is_trees_due {
      return Some(Self::offer(
        index,
        cascade,
        frame,
        scheduler,
        CascadeRequest {
          draw: CascadeDraw::Kept,
          fit: cascade.drawn_fit,
          is_urgent: true,
        },
      ));
    }

    let is_wanted: bool = is_trees_due || has_moving || cascade.has_drawn_moving;

    if !is_wanted {
      cascade.waited = 0;

      return None;
    }

    let (draw, fit): (CascadeDraw, SunCascade) = if is_held {
      (CascadeDraw::Swaying, static_fit)
    } else {
      (CascadeDraw::Static, current)
    };
    let is_urgent: bool = has_moving || cascade.has_drawn_moving;

    Some(Self::offer(
      index,
      cascade,
      frame,
      scheduler,
      CascadeRequest { draw, fit, is_urgent },
    ))
  }

  /// Offers the draw a cascade asks for (`request`): at once for the nearest, one never drawn, staggering off, one whose
  /// box strayed past its map, or an urgent one; else within its stagger.
  fn offer(
    index: usize,
    cascade: &ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
    CascadeRequest { draw, fit, is_urgent }: CascadeRequest,
  ) -> CascadeOffer {
    let (passes, seed): (&'static [&'static str], f32) = match draw {
      CascadeDraw::Whole => (&WHOLE_PASSES[index], WHOLE_SEEDS[index]),
      CascadeDraw::Static => (&STATIC_PASSES[index], WHOLE_SEEDS[index]),
      CascadeDraw::Swaying => (&SWAY_PASSES[index], SWAY_SEEDS[index]),
      CascadeDraw::Kept => (&KEPT_PASSES[index], KEPT_SEED),
      CascadeDraw::Banded(_) => (&SWAY_PASSES[index], SWAY_SEEDS[index] / BANDS as f32),
    };
    // The nearest is seen closest, and a cascade never drawn or strayed past its map shows wrong until drawn.
    let is_now: bool = index == 0
      || is_urgent
      || cascade.drawn.is_none()
      || !frame.settings.is_staggered
      || !cascade.cascade.is_held_by(&cascade.drawn_fit);
    let job: PeriodicJob = if is_now {
      PeriodicJob::new_now(passes, seed, 1.0)
    } else {
      PeriodicJob::new_waiting(passes, seed, 1.0, cascade.waited, 1 << index)
    };

    CascadeOffer {
      index,
      ticket: scheduler.offer(job),
      draw,
      fit,
      contents: frame.scene.get_contents(),
    }
  }

  /// Offers a band of a far cascade's next static layer, drawn ahead once its fit strayed past `PREFETCH_HOLD` of what
  /// its static layer holds, at the fit it strayed to; started again where the fit strays as far from that or the scene
  /// changes under it. None while its static layer holds well, or is drawn again whole this frame.
  fn offer_band(
    index: usize,
    cascade: &mut ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
  ) -> Option<PeriodicTicket> {
    let scene = frame.scene;
    let current: SunCascade = cascade.cascade;
    let is_static_drawn: bool = cascade.drawn_static.is_some() && current.is_held_by(&cascade.static_fit);

    if !is_static_drawn || current.is_held_within(&cascade.static_fit, PREFETCH_HOLD) {
      cascade.prefetch = None;

      return None;
    }

    let started: ShadowPrefetch = ShadowPrefetch {
      fit: current,
      contents: scene.get_contents(),
      bands: 0,
      waited: 0,
    };
    let prefetch: &mut ShadowPrefetch = cascade.prefetch.get_or_insert(started);
    let frustum: CameraFrustum = prefetch.fit.view.get_frustum();

    if !current.is_held_within(&prefetch.fit, PREFETCH_HOLD)
      || scene.is_changed_since(prefetch.contents, |sphere| frustum.reaches(sphere))
    {
      *prefetch = ShadowPrefetch {
        waited: prefetch.waited,
        ..started
      };
    }

    let job: PeriodicJob = PeriodicJob::new_waiting(
      &BAND_PASSES[index],
      WHOLE_SEEDS[index] / BANDS as f32,
      1.0,
      prefetch.waited,
      BAND_WAIT_MOST,
    );

    Some(scheduler.offer(job))
  }

  /// How far in from its map's edge a cascade drawn by `projection` holds a point, past the edge the sun's pass keeps
  /// off (`EDGE` in `common/sun_shadow.wgsl`), as a share of the map; none where it does not hold it.
  fn get_cascade_inset(projection: Mat4, point: Vec3) -> Option<f32> {
    let clip: Vec4 = projection * point.extend(1.0);
    let (u, v): (f32, f32) = (clip.x * 0.5 + 0.5, clip.y * -0.5 + 0.5);
    let inset: f32 = u.min(v).min(1.0 - u).min(1.0 - v) - SUN_SHADOW_EDGE;

    (inset > 0.0 && clip.z > 0.0 && clip.z < 1.0).then_some(inset)
  }

  /// Each cascade's blend band, the settings' `blend` widened to `NEAR_BLEND` for the nearest.
  pub(crate) fn get_cascade_blends(blend: f32) -> Vec4 {
    Vec4::from_array(std::array::from_fn(|index| {
      if index < NEAR_BLENDED {
        blend.max(NEAR_BLEND)
      } else {
        blend
      }
    }))
  }

  /// Points of the sphere `reach` metres about the `camera` that it sees, where the shadows of what stands within it
  /// land: along rays through a grid over the screen, its edges included, at shares of the reach, so a view of any
  /// width samples it. What the camera does not see casts onto nothing it shows, and no cascade's box reaches behind it.
  pub(crate) fn sample_seen(camera: &CameraView, reach: f32) -> Vec<Vec3> {
    let inverse: Mat4 = camera.get_view_projection().inverse();
    let last: f32 = (SEEN_GRID - 1) as f32;

    (0..SEEN_GRID * SEEN_GRID)
      .filter_map(|cell| {
        let x: f32 = (cell % SEEN_GRID) as f32 / last * 2.0 - 1.0;
        let y: f32 = (cell / SEEN_GRID) as f32 / last * 2.0 - 1.0;

        (inverse.project_point3(Vec3::new(x, y, 0.5)) - camera.position).try_normalize()
      })
      .flat_map(|ray| SEEN_SHARES.map(|share| camera.position + ray * reach * share))
      .collect()
  }

  /// Shows a cascade's static layer drawn ahead in place of the one shown, once every band of it is drawn.
  fn swap_prefetched(index: usize, cascade: &mut ShadowCascadeView) {
    let Some(prefetch) = cascade.prefetch.filter(|prefetch| prefetch.bands == BANDS) else {
      return;
    };

    cascade.static_layer = Self::get_hidden_layer(index, cascade.static_layer);
    cascade.static_fit = prefetch.fit;
    cascade.drawn_static = Some((prefetch.fit.version, prefetch.contents));
    cascade.prefetch = None;
    cascade.is_swapped = true;
  }

  /// The static layer a far cascade draws ahead into, the one of its two it does not show.
  pub(super) fn get_hidden_layer(index: usize, shown: u32) -> u32 {
    let second: u32 = (RENDER_MAX_SHADOW_CASCADES + index - BANDED_FROM) as u32;

    if shown == index as u32 { second } else { index as u32 }
  }

  /// A band's region of a static layer `resolution` texels across: every column of its rows, the bands together
  /// covering every row.
  pub(super) fn get_band_region(band: u32, resolution: u32) -> [u32; 4] {
    let (first, last): (u32, u32) = (band * resolution / BANDS, (band + 1) * resolution / BANDS);

    [0, first, resolution, last - first]
  }
}
