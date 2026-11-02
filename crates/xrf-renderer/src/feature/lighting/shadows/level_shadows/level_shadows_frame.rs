use glam::{Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::lighting::shadows::cascades::shadow_cascade_view::ShadowCascadeView;
use crate::feature::lighting::shadows::cascades::sun_shadow_maps::SunShadowMaps;
use crate::feature::lighting::shadows::level_shadows::{
  BANDS, CascadeDraw, LandedCasters, LevelShadows, STATIC_LAYERS,
};
use crate::feature::lighting::shadows::shadow_cut::ShadowCut;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::scene::level::shadow::shadow_frame::ShadowFrame;

impl LevelShadows {
  /// Fits the cascades, offers those wanting a draw to `scheduler` while the sun lights anything, and takes what the
  /// sun samples them by.
  pub fn prepare_cascades(
    &mut self,
    device: &wgpu::Device,
    view_layout: &wgpu::BindGroupLayout,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
  ) {
    let settings = frame.settings;
    let count: usize = settings.get_cascade_count();

    self.values.count = count as u32;

    // A cascade past those asked for holds nothing, drawn afresh once it is asked for again.
    for (index, cascade) in self.cascades.iter_mut().enumerate().skip(count) {
      cascade.release(index);
    }

    self.offers.clear();
    self.band_offers.clear();
    self.due.clear();
    self.bands.clear();

    self.per_frame.clear();
    self
      .per_frame
      .extend(frame.casters.iter().map(|&casters| LandedCasters {
        casters,
        cascades: [false; RENDER_MAX_SHADOW_CASCADES],
      }));
    self.values.blends = Self::get_cascade_blends(settings.blend);

    // The static layers pay only while the trees sway or anything moves every frame: still, a cascade draws again only
    // where its static layer would.
    let is_moving: bool = frame.sway.amplitude * frame.sway.reach > 0.0 || !frame.casters.is_empty();

    match (&self.maps, is_moving && count > 0) {
      (Some(maps), true) if self.statics.is_none() => {
        self.statics = Some(SunShadowMaps::new_static(device, maps.resolution, STATIC_LAYERS));
      }
      (_, false) if self.statics.is_some() => {
        self.statics = None;

        for (index, cascade) in self.cascades.iter_mut().enumerate() {
          cascade.forget_static(index);
        }
      }
      _ => {}
    }

    let kept_count: usize = frame
      .casters
      .iter()
      .map(|casters| Self::get_reached_count(settings, casters.reach))
      .max()
      .unwrap_or(0);

    match (&self.maps, self.statics.is_some() && kept_count > 0) {
      (Some(maps), true) if self.kept.as_ref().is_none_or(|kept| kept.layers.len() != kept_count) => {
        self.kept = Some(SunShadowMaps::new_kept(device, maps.resolution, kept_count as u32));

        for cascade in &mut self.cascades {
          cascade.is_kept = false;
        }
      }
      (_, false) if self.kept.is_some() => {
        self.kept = None;

        for cascade in &mut self.cascades {
          cascade.is_kept = false;
        }
      }
      _ => {}
    }

    // A sun of no colour lights nothing its shadow could take away: no cascade is drawn, the maps keeping their last.
    if count > 0 && frame.is_sunlit {
      self.offer_cascades(device, view_layout, frame, count, scheduler);
    }

    let look: Vec3 = -frame.camera.view.inverse().z_axis.truncate();

    self.values.forward = look.extend(0.0);
    self.values.filter_reach = settings.get_filter();
    self.values.resolution = self.maps.as_ref().map_or(1, |maps| maps.resolution) as f32;
    self.values.bias = settings.bias;
  }

  /// Readies the cascades and bands `scheduler` granted to be culled and drawn, marking each drawn; one not granted
  /// waits a frame more. `encoder` takes what readying them writes.
  pub fn commit_cascades(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let resolution: u32 = self.maps.as_ref().map_or(1, |maps| maps.resolution);
    let list_bytes: u64 = scene.cull.get_list_bytes();
    let ready = |cut: &mut ShadowCut, encoder: &mut FrameEncoder, view: &CameraView, rect| {
      cut.reserve_lists(device, list_bytes);
      scene.cull.reset_args(encoder, &cut.args);
      cut.view.write(encoder, &CameraUniform::new(view, rect, Vec4::ZERO));
    };

    for offer in std::mem::take(&mut self.offers) {
      let has_moving: bool = self.has_moving(offer.index);
      let cascade: &mut ShadowCascadeView = &mut self.cascades[offer.index];

      if !scheduler.is_granted(offer.ticket) {
        cascade.waited += 1;

        continue;
      }

      match offer.draw {
        CascadeDraw::Banded(band) => {
          let [x, y, width, height] = Self::get_band_region(band, resolution);

          ready(
            &mut cascade.cut,
            encoder,
            &offer.fit.get_rows_view((y, y + height), resolution),
            RenderRect {
              x: x as i32,
              y: y as i32,
              width,
              height,
            },
          );
        }
        _ => ready(
          &mut cascade.cut,
          encoder,
          &offer.fit.view,
          RenderRect {
            x: 0,
            y: 0,
            width: resolution,
            height: resolution,
          },
        ),
      }

      self.due.push((offer.index, offer.draw));
      cascade.drawn = Some((offer.fit.version, offer.contents));

      // A kept layer's copy shows the trees as last drawn, so their interval runs on from that draw; a banded sweep's
      // from its start, once its last band is drawn.
      match offer.draw {
        CascadeDraw::Kept => {}
        CascadeDraw::Banded(band) => {
          let started: f32 = cascade.sway_sweep.map_or(frame.sway.time, |(started, _)| started);

          cascade.sway_sweep = (band + 1 < BANDS).then_some((started, band + 1));

          if cascade.sway_sweep.is_none() {
            cascade.drawn_at = started;
          }
        }
        _ => {
          cascade.drawn_at = frame.sway.time;
          cascade.sway_sweep = None;
        }
      }

      cascade.drawn_fit = offer.fit;
      cascade.waited = 0;
      cascade.is_swapped = false;
      cascade.has_drawn_moving = offer.draw != CascadeDraw::Whole && has_moving;
      cascade.is_kept = offer.draw != CascadeDraw::Whole && self.kept.is_some() && has_moving;

      // Trees drawn again with each sway, or into a kept layer, follow the camera: they are drawn only where their
      // shadows can fall in its view, and again once it turns away.
      if matches!(offer.draw, CascadeDraw::Static | CascadeDraw::Swaying) {
        cascade.received_at = (cascade.is_kept || frame.settings.is_swaying(offer.index))
          .then(|| (frame.camera.position, frame.camera.get_look()));
      } else if offer.draw == CascadeDraw::Whole {
        cascade.received_at = None;
      }

      if offer.draw == CascadeDraw::Static {
        cascade.drawn_static = Some((offer.fit.version, offer.contents));
        cascade.static_fit = offer.fit;
        cascade.prefetch = None;
      }

      self.values.matrices[offer.index] = offer.fit.view.get_view_projection();
      self.values.texels[offer.index] = offer.fit.texel;
    }

    for (index, ticket) in std::mem::take(&mut self.band_offers) {
      let cascade: &mut ShadowCascadeView = &mut self.cascades[index];
      let (Some(prefetch), Some(cut)) = (cascade.prefetch.as_mut(), cascade.band.as_mut()) else {
        continue;
      };

      if !scheduler.is_granted(ticket) {
        prefetch.waited += 1;

        continue;
      }

      let [x, y, width, height] = Self::get_band_region(prefetch.bands, resolution);

      ready(
        cut,
        encoder,
        &prefetch.fit.get_rows_view((y, y + height), resolution),
        RenderRect {
          x: x as i32,
          y: y as i32,
          width,
          height,
        },
      );

      self.bands.push((
        index,
        prefetch.bands,
        Self::get_hidden_layer(index, cascade.static_layer),
      ));
      prefetch.bands += 1;
      prefetch.waited = 0;
    }
  }
}
