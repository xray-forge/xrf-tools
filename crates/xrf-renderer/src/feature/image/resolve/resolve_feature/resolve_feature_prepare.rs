use std::time::{Duration, Instant};

use glam::{Mat4, Vec2};
use xrf_dlss::{Dlss, DlssPreset, DlssQuality, DlssRayPreset};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::image::resolve::render_antialiasing::RenderAntialiasing;
use crate::contract::feature::image::resolve::render_upscaling_settings::RenderUpscalingSettings;
use crate::feature::image::exposure::reading::pre_exposure::PreExposure;
use crate::feature::image::resolve::dlss::dlss_resolve::DlssResolve;
use crate::feature::image::resolve::dlss::ray_reconstruction_resolve::RayReconstructionResolve;
use crate::feature::image::resolve::fsr::fsr_dispatch::FsrDispatch;
use crate::feature::image::resolve::fsr::fsr_uniform::FsrUniform;
use crate::feature::image::resolve::fsr::targets::fsr_targets::FsrTargets;
use crate::feature::image::resolve::resolve::Resolve;
use crate::feature::image::resolve::resolve_decisions::ResolveDecisions;
use crate::feature::image::resolve::resolve_feature::{DlssKey, RayKey, ResolveFeature};
use crate::feature::image::resolve::temporal::temporal_frame::TemporalFrame;
use crate::feature::image::resolve::temporal::temporal_pass::TemporalPass;
use crate::feature::image::resolve::temporal::temporal_resolve::TemporalResolve;
use crate::feature::image::resolve::temporal::temporal_uniform::TemporalUniform;
use crate::feature::image::resolve::upscale::upscale_pass::UpscalePass;
use crate::feature::image::resolve::upscale::upscale_uniform::UpscaleUniform;
use crate::frame::targets::frame_texture::FrameTexture;
use crate::frame::targets::history::History;
use crate::frame::targets::view_targets::ViewTargets;
use crate::scene::level::view::view_prepare::ViewPrepare;

impl ResolveFeature {
  /// Takes this frame's resolve for targets `render` texels across, none before they are made, resolved and shown as
  /// the `frame` asks: the temporal resolve's history made or kept and what it reads, and the frame at the viewport's
  /// size while it is drawn or resolved at another. DLSS reconstructs rays where asked, the GPU has it and the scene
  /// holds linear light, with the model the light asks; else it upscales alone. The light is written at
  /// `pre_exposure`, which each resolve carries its history by. Answers what the units prepared after it read.
  pub fn prepare(
    &mut self,
    frame: &ViewPrepare<'_>,
    now: Instant,
    render: Option<(u32, u32)>,
    pre_exposure: PreExposure,
  ) -> ResolveDecisions {
    let &ViewPrepare {
      device,
      dlss,
      view,
      resolved,
      output,
      options,
      ..
    } = frame;
    let upscaling: RenderUpscalingSettings = options.output.upscaling;
    let is_linear: bool = options.is_scene_linear();
    let is_light_physical: bool = options.is_light_physical();

    self.upscaling = upscaling;
    self.is_light_physical = is_light_physical;
    self.pre_exposure = pre_exposure;

    self.frame_time = self
      .prepared_at
      .map_or(Duration::ZERO, |before| now.duration_since(before));
    self.prepared_at = Some(now);

    // A resolve's frames turn the screen-space effects' noise on; a reference's sample set it (`next_jitter`).
    if self.temporal.is_some() && self.reference.is_none() {
      self.noise_frame = self.noise_frame.wrapping_add(1);
    }

    let display: (u32, u32) = (resolved.0.max(1), resolved.1.max(1));
    let viewport: (u32, u32) = (output.width.max(1), output.height.max(1));

    self.resolved = display;

    if self.is_cut(view)
      && let Some(resolve) = &mut self.resolve
    {
      resolve.reset();
    }

    match (render, self.temporal) {
      (Some(render), Some(RenderAntialiasing::Fsr)) => {
        self.prepare_fsr(frame, (render, display));
      }
      (Some(render), Some(RenderAntialiasing::Dlss)) => {
        let is_reconstructing: bool =
          upscaling.is_ray_reconstructed && is_linear && dlss.is_some_and(Dlss::has_ray_reconstruction);

        if !(is_reconstructing && self.prepare_ray_reconstruction(frame, (render, display)))
          && !self.prepare_dlss(frame, (render, display))
        {
          self.temporal = Some(RenderAntialiasing::Fsr);
          self.prepare_fsr(frame, (render, display));
        }
      }
      (Some(render), Some(_)) => self.prepare_temporal(device, view, (render, display)),
      _ => self.resolve = None,
    }

    let Some(render) = render else {
      return self.get_decisions();
    };
    // What reaches the viewport's rescale: what the temporal resolve resolved, or the frame as drawn.
    let source: (u32, u32) = if self.resolve.is_some() { display } else { render };

    self.rescale = Self::get_rescale(source, viewport);
    // Sharpened where upscaled, and where a temporal resolve gathers frames at the viewport's size, whose filtering and
    // resampled history soften what moves as an upscale does; never an FSR debug view, which shows its values as they
    // stand.
    self.is_sharpened =
      upscaling.sharpening > 0.0 && (render.0 < viewport.0 || self.resolve.is_some()) && self.fsr_view.is_none();

    if render == viewport && self.rescale.is_none() && !self.is_sharpened {
      self.upscaled = None;
    } else if self
      .upscaled
      .as_ref()
      .is_none_or(|[upscaled, _]| upscaled.get_size() != viewport)
    {
      self.upscaled = Some(["upscaled", "sharpened"].map(|label| {
        FrameTexture::new(
          device,
          label,
          viewport,
          UpscalePass::FORMAT,
          wgpu::TextureUsages::COPY_DST,
        )
      }));
    }

    self.upscale = UpscaleUniform {
      output_size: Vec2::new(viewport.0 as f32, viewport.1 as f32),
      sharpness: upscaling.get_sharpness(),
      linear: f32::from(u8::from(is_linear)),
    };

    self.get_decisions()
  }

  fn get_decisions(&self) -> ResolveDecisions {
    ResolveDecisions {
      noise_frame: self.noise_frame,
      reference_sample: self.get_reference_sample(),
      is_ray_reconstructed: self.is_ray_reconstructed(),
    }
  }

  /// TAA's history at the output's size, kept while it still fits, and what it reads: this frame's view projection,
  /// the last one's, and the jitter.
  fn prepare_temporal(
    &mut self,
    device: &wgpu::Device,
    view: &CameraView,
    (render, display): ((u32, u32), (u32, u32)),
  ) {
    let is_fitting: bool = matches!(
      &self.resolve,
      Some(Resolve::Temporal(temporal))
        if temporal.render == render && temporal.history.get_written().get_size() == display
    );

    if !is_fitting {
      self.resolve = Some(Resolve::Temporal(Box::new(TemporalResolve {
        history: History::new(|side| {
          FrameTexture::new(
            device,
            ["temporal history 0", "temporal history 1"][side],
            display,
            TemporalPass::HISTORY,
            wgpu::TextureUsages::COPY_SRC,
          )
        }),
        flicker: History::new(|side| {
          FrameTexture::new(
            device,
            ["temporal flicker 0", "temporal flicker 1"][side],
            display,
            TemporalPass::FLICKER,
            wgpu::TextureUsages::empty(),
          )
        }),
        render,
        previous: None,
        uniform: TemporalUniform::default(),
      })));
    }

    let Some(Resolve::Temporal(temporal)) = &mut self.resolve else {
      return;
    };
    let TemporalResolve {
      history,
      previous,
      uniform,
      ..
    } = &mut **temporal;
    let current: Mat4 = view.get_view_projection();
    let (last, last_view): (Mat4, Mat4) = previous.unwrap_or((current, view.view));

    *uniform = TemporalUniform::new(TemporalFrame {
      current,
      previous: last,
      previous_view: last_view,
      jitter: self.offset,
      is_history_valid: history.is_valid() && previous.is_some(),
      history_scale: self.pre_exposure.get_history_scale(),
    });
    *previous = Some((current, view.view));
  }

  /// FSR's targets, kept while they still fit, and its constants, over colour of linear light where the scene holds it.
  fn prepare_fsr(
    &mut self,
    &ViewPrepare {
      device, view, options, ..
    }: &ViewPrepare<'_>,
    (render, display): ((u32, u32), (u32, u32)),
  ) {
    let is_hdr: bool = options.is_scene_linear();

    if !matches!(&self.resolve, Some(Resolve::Fsr { targets, .. }) if targets.is_sized(render, display)) {
      self.resolve = Some(Resolve::Fsr {
        targets: Box::new(FsrTargets::new(device, render, display, ViewTargets::SCENE)),
        uniform: FsrUniform::default(),
      });
    }

    let Some(Resolve::Fsr { targets, uniform }) = &mut self.resolve else {
      return;
    };

    // The frame before's jitter, turned back from FSR's sense; none read on a fresh frame.
    let previous_offset: Vec2 = -uniform.jitter;

    *uniform = FsrUniform::new(
      FsrDispatch {
        render,
        display,
        jitter: self.offset,
        previous_jitter: previous_offset,
        phases: self.phases,
        frame_index: targets.frames.get_frame(),
        is_hdr,
      },
      view,
    )
    .with_delta_pre_exposure(self.pre_exposure.get_history_scale());
  }

  /// DLSS's upscaler, kept while it still fits, upscaling targets `render` texels across into `display` pixels over
  /// colour of linear light where the scene holds it; whether it resolves this frame, which FSR does where DLSS refuses
  /// them.
  fn prepare_dlss(
    &mut self,
    &ViewPrepare {
      device, dlss, options, ..
    }: &ViewPrepare<'_>,
    (render, display): ((u32, u32), (u32, u32)),
  ) -> bool {
    let is_hdr: bool = options.is_scene_linear();
    let model: (DlssQuality, DlssPreset) = (
      self.upscaling.scale.get_dlss_quality(),
      self.upscaling.dlss_preset.to_dlss(),
    );
    let key: DlssKey = (display, model, is_hdr);

    if self.refused == Some(key) {
      return false;
    }

    let Some(dlss) = dlss else {
      return false;
    };

    if !matches!(&self.resolve, Some(Resolve::Dlss(resolve)) if resolve.upscaler.is_fitting(display, model, is_hdr)) {
      // NGX makes and releases its upscalers with the device idle: the last one goes first.
      self.resolve = None;

      match dlss.create_upscaler(display, model, is_hdr) {
        Ok(upscaler) => {
          self.resolve = Some(Resolve::Dlss(Box::new(DlssResolve {
            upscaler,
            output: FrameTexture::new(
              device,
              "dlss output",
              display,
              DlssResolve::FORMAT,
              // NGX writes the output as storage and clears it with a transfer command.
              wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            ),
            is_reset: true,
          })));
        }
        Err(error) => {
          log::warn!("FSR resolves in DLSS's place: {error}");
          self.refused = Some(key);

          return false;
        }
      }
    }

    let Some(Resolve::Dlss(resolve)) = &self.resolve else {
      return false;
    };
    let (min, max) = resolve.upscaler.get_render_range();

    if !((min.0..=max.0).contains(&render.0) && (min.1..=max.1).contains(&render.1)) {
      log::warn!(
        "FSR resolves in DLSS's place: DLSS upscales from {min:?} to {max:?} into {display:?}, not from {render:?}"
      );
      self.refused = Some(key);

      return false;
    }

    true
  }

  /// DLSS's ray reconstructor, kept while it still fits, reconstructing targets `render` texels across into `display`
  /// pixels, and this frame's view and projection with the camera's jitter taken out; whether it resolves this frame,
  /// which super resolution does where it refuses them.
  fn prepare_ray_reconstruction(
    &mut self,
    &ViewPrepare { device, dlss, view, .. }: &ViewPrepare<'_>,
    (render, display): ((u32, u32), (u32, u32)),
  ) -> bool {
    let quality: DlssQuality = self.upscaling.scale.get_dlss_quality();
    let preset: DlssRayPreset = self.upscaling.ray_reconstruction_preset.to_dlss(self.is_light_physical);
    let key: RayKey = (display, quality, preset);

    if self.refused_rays == Some(key) {
      return false;
    }

    let Some(dlss) = dlss else {
      return false;
    };
    let is_fitting: bool = matches!(
      &self.resolve,
      Some(Resolve::RayReconstruction(resolve)) if resolve.reconstructor.is_fitting(display, (quality, preset))
    );

    if !is_fitting {
      // NGX makes and releases its features with the device idle: the last one goes first.
      self.resolve = None;

      match dlss.create_ray_reconstructor(display, (quality, preset)) {
        Ok(reconstructor) => {
          self.resolve = Some(Resolve::RayReconstruction(Box::new(RayReconstructionResolve {
            reconstructor,
            output: FrameTexture::new(
              device,
              "ray reconstruction output",
              display,
              DlssResolve::FORMAT,
              // As super resolution's: NGX writes it as storage and clears it with a transfer command.
              wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            ),
            is_reset: true,
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
          })));
        }
        Err(error) => {
          log::warn!("DLSS upscales without reconstructing rays: {error}");
          self.refused_rays = Some(key);

          return false;
        }
      }
    }

    let Some(Resolve::RayReconstruction(resolve)) = &mut self.resolve else {
      return false;
    };
    let (min, max) = resolve.reconstructor.get_render_range();

    if !((min.0..=max.0).contains(&render.0) && (min.1..=max.1).contains(&render.1)) {
      log::warn!(
        "DLSS upscales without reconstructing rays: it reconstructs from {min:?} to {max:?} into {display:?}, not from {render:?}"
      );
      self.refused_rays = Some(key);

      return false;
    }

    (resolve.view, resolve.projection) = Self::get_ray_camera(view);

    true
  }

  /// The view and projection DLSS ray reconstruction places hits through: the frame's `view` as prepared, unjittered,
  /// since NGX takes the jitter apart.
  pub fn get_ray_camera(view: &CameraView) -> (Mat4, Mat4) {
    (view.view, view.projection)
  }
}
