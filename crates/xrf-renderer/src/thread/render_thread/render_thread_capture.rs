use std::collections::BTreeMap;

use xrf_error::XrfError;
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureAccess, GraphTextureDescriptor,
  UniformBinding,
};

use crate::capture::burst::burst_job::BurstJob;
use crate::capture::burst::burst_shot::BurstShot;
use crate::capture::capture_job::CaptureJob;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::pass::overlay_draw::OverlayDraw;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::overlay_pass::OverlayPass;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::present_pass::PresentPass;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::scene_view::SceneView;
use crate::thread::gpu_state::GpuState;
use crate::viewport::pending_screenshot::PendingScreenshot;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::window_frame::WindowFrame;

use super::{BurstCopy, RenderThread, ShotCopy, ViewFrame};

impl RenderThread {
  /// The captures this frame copies out of their window's image once it is composed, before it is presented, and the
  /// screenshots it draws apart, each read once back. A viewport gives one of each a frame, the first asked, so those
  /// asked together land on consecutive frames.
  pub(super) fn take_captures(
    &mut self,
    device: &wgpu::Device,
    windows: &[WindowFrame],
  ) -> Vec<(RenderViewportId, FrameCapture, CaptureReply)> {
    let mut captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)> = Vec::new();

    for window in windows {
      for (id, rect, shown) in &window.drawn {
        // A viewport capturing draws no frame of its own to capture; what it was asked waits.
        let Some(viewport) = self.viewports.get_mut(id).filter(|viewport| !viewport.is_capturing()) else {
          continue;
        };

        if !viewport.captures.is_empty() {
          let reply: CaptureReply = viewport.captures.remove(0);

          match FrameCapture::new(device, window.format, *shown, viewport.frame) {
            Ok(capture) => captures.push((*id, capture, reply)),
            Err(error) => {
              let _ = reply.send(Err(error));
            }
          }
        }

        if !viewport.screenshots.is_empty() {
          let PendingScreenshot { screenshot, reply } = viewport.screenshots.remove(0);

          match FrameCapture::new_screenshot(device, window.format, (rect.width, rect.height), viewport.frame) {
            Ok(capture) => captures.push((*id, capture.with_screenshot(screenshot), reply)),
            Err(error) => {
              let _ = reply.send(Err(error));
            }
          }
        }
      }
    }

    captures
  }

  /// The captures' shots this frame copies out, each on the last of its settling frames, in its window's format; a
  /// shot that cannot be copied stops its capture.
  pub(super) fn take_capture_shots(&mut self, device: &wgpu::Device, windows: &[WindowFrame]) -> Vec<ShotCopy> {
    let mut shots: Vec<ShotCopy> = Vec::new();

    for window in windows {
      for (id, ..) in &window.drawn {
        let Some(viewport) = self.viewports.get_mut(id) else {
          continue;
        };
        let Some(shot) = viewport
          .capture
          .as_ref()
          .filter(|capture| !capture.is_ended() && capture.is_kept_frame())
          .and_then(CaptureJob::get_drawn_shot)
        else {
          continue;
        };

        match FrameCapture::new_shot(
          device,
          window.format,
          shot.get_drawn_size(),
          viewport.frame,
          shot.is_covered(),
        ) {
          Ok(capture) => shots.push((*id, capture)),
          Err(error) => viewport.stop_capture(&error.to_string()),
        }
      }
    }

    shots
  }

  /// The bursts' frames this frame copies out, each one read back drawn by a viewport readied this frame (`frames`), in
  /// its window's format at the viewport's whole size; a frame that cannot be copied stops its burst.
  pub(super) fn take_burst_shots(
    &mut self,
    device: &wgpu::Device,
    windows: &[WindowFrame],
    frames: &[ViewFrame],
  ) -> Vec<BurstCopy> {
    let mut shots: Vec<BurstCopy> = Vec::new();

    for window in windows {
      for (id, rect, _) in &window.drawn {
        let Some(viewport) = self
          .viewports
          .get_mut(id)
          .filter(|_| frames.iter().any(|(drawn, ..)| drawn == id))
        else {
          continue;
        };
        let (Some(frame), Some(material)) = (
          viewport
            .burst
            .as_ref()
            .filter(|burst| burst.is_drawing())
            .and_then(BurstJob::get_kept_frame),
          viewport.shown.as_ref().and_then(|(_, view)| view.get_render_size()),
        ) else {
          continue;
        };

        match BurstShot::new(
          device,
          (window.format, (rect.width, rect.height), viewport.frame),
          material,
          frame,
        ) {
          Ok(shot) => shots.push((*id, shot)),
          Err(error) => viewport.stop_burst(&error.to_string()),
        }
      }
    }

    shots
  }

  /// Puts captures a window gave no image for back at the front of their viewport's, for its next presented frame;
  /// answers one whose viewport is gone.
  pub(super) fn retry_captures(&mut self, retried: Vec<(RenderViewportId, CaptureReply)>) {
    for (id, reply) in retried.into_iter().rev() {
      match self.viewports.get_mut(&id) {
        Some(viewport) => viewport.captures.insert(0, reply),
        None => {
          let _ = reply.send(Err(XrfError::new_invalid_error(format!(
            "Viewport {} is no longer drawn",
            id.0
          ))));
        }
      }
    }
  }

  /// Declares a viewport's screenshot: its frame (`output`) presented into a texture of its own at its whole size, as
  /// its window shows it but without its overlays or its selection's outline, its helpers drawn over it where it asks,
  /// then copied out.
  pub(super) fn declare_screenshot<'a>(
    (graph, runtime): (&mut FrameGraph<'a>, &mut GraphRuntime),
    gpu: &'a GpuState,
    viewports: &'a BTreeMap<RenderViewportId, RenderViewport>,
    (id, capture): (RenderViewportId, &'a FrameCapture),
    output: Option<&SceneOutput>,
  ) {
    let Some(viewport) = viewports.get(&id) else {
      return;
    };
    let (Some(output), Some(binding), Some((_, view))) = (output, viewport.binding.as_ref(), viewport.shown.as_ref())
    else {
      return;
    };
    let texture: GraphTexture = Self::declare_finished_frame(
      graph,
      runtime,
      FinishedFrame {
        gpu,
        view,
        binding,
        output,
        label: "screenshot",
        capture,
        helpers: capture.is_helped().then(|| view.get_helper_draws()),
      },
    );

    graph
      .add_encoder_pass("screenshot copy")
      .texture(texture, GraphTextureAccess::CopySource)
      .keep()
      .record(move |context| {
        let texture: &wgpu::Texture = context.get_texture(texture).texture;

        capture.encode(context.get_encoder(), texture);
      });
  }

  /// Declares a capture's shot: the frame its capture drew (`output`) presented into a texture of its own at the
  /// shot's drawn size, then copied out with its depth, which a map tile's coverage is read from.
  pub(super) fn declare_capture_shot<'a>(
    (graph, runtime): (&mut FrameGraph<'a>, &mut GraphRuntime),
    gpu: &'a GpuState,
    viewports: &'a BTreeMap<RenderViewportId, RenderViewport>,
    (id, capture): (RenderViewportId, &'a FrameCapture),
    output: Option<&SceneOutput>,
  ) {
    let (Some(output), Some((_, view, binding))) =
      (output, viewports.get(&id).and_then(RenderViewport::get_drawn_view))
    else {
      return;
    };
    let texture: GraphTexture = Self::declare_finished_frame(
      graph,
      runtime,
      FinishedFrame {
        gpu,
        view,
        binding,
        output,
        label: "capture shot",
        capture,
        helpers: None,
      },
    );
    let depth: GraphTexture = output.targets.depth;

    graph
      .add_encoder_pass("capture shot copy")
      .texture(texture, GraphTextureAccess::CopySource)
      .texture(depth, GraphTextureAccess::CopySource)
      .keep()
      .record(move |context| {
        let picture: &wgpu::Texture = context.get_texture(texture).texture;
        let depth: &wgpu::Texture = context.get_texture(depth).texture;
        let encoder: &mut wgpu::CommandEncoder = context.get_encoder();

        capture.encode(encoder, picture);
        capture.encode_coverage(encoder, depth);
      });
  }

  /// Declares a burst's frame: its viewport's frame (`output`) presented into a texture of its own at the viewport's
  /// whole size, as a screenshot is, then copied out with the G-buffer's material target, whose alpha holds the marks.
  pub(super) fn declare_burst_shot<'a>(
    (graph, runtime): (&mut FrameGraph<'a>, &mut GraphRuntime),
    gpu: &'a GpuState,
    viewports: &'a BTreeMap<RenderViewportId, RenderViewport>,
    (id, shot): (RenderViewportId, &'a BurstShot),
    output: Option<&SceneOutput>,
  ) {
    let Some(viewport) = viewports.get(&id) else {
      return;
    };
    let (Some(output), Some(binding), Some((_, view))) = (output, viewport.binding.as_ref(), viewport.shown.as_ref())
    else {
      return;
    };
    let texture: GraphTexture = Self::declare_finished_frame(
      graph,
      runtime,
      FinishedFrame {
        gpu,
        view,
        binding,
        output,
        label: "burst frame",
        capture: shot.get_picture(),
        helpers: None,
      },
    );
    let material: GraphTexture = output.targets.material;

    graph
      .add_encoder_pass("burst frame copy")
      .texture(texture, GraphTextureAccess::CopySource)
      .texture(material, GraphTextureAccess::CopySource)
      .keep()
      .record(move |context| {
        let picture: &wgpu::Texture = context.get_texture(texture).texture;
        let material: &wgpu::Texture = context.get_texture(material).texture;
        let encoder: &mut wgpu::CommandEncoder = context.get_encoder();

        shot.get_picture().encode(encoder, picture);
        shot.encode_marks(encoder, material);
      });
  }

  /// Declares `frame`'s view's finished frame presented into a texture of its own, at its capture's size and in its
  /// format, without overlays or the selection's outline, and with its helpers' marks and fading and their wires and
  /// icons over it where it gives them; answers the texture, for its copy.
  fn declare_finished_frame<'a>(
    graph: &mut FrameGraph<'a>,
    runtime: &mut GraphRuntime,
    frame: FinishedFrame<'a, '_>,
  ) -> GraphTexture {
    let FinishedFrame {
      gpu,
      view,
      binding,
      output,
      label,
      capture,
      helpers,
    } = frame;
    let size: (u32, u32) = capture.get_size();
    let format: wgpu::TextureFormat = capture.get_format();
    let shown: PresentUniform = match helpers {
      Some(_) => view.get_present_uniform().for_helped_screenshot(size),
      None => view.get_present_uniform().for_screenshot(size),
    };
    let uniform: UniformBinding<PresentUniform> = runtime.push_uniform(&shown);
    let parameters: PresentParameters<'a> = PresentParameters {
      present: uniform,
      ..Self::to_present_parameters(gpu, output)
    };
    // The helpers are read against the frame's depth, placed by the capture's own size, as the window's are.
    let overlay_parameters: Option<OverlayParameters> = helpers.map(|_| OverlayParameters {
      depth_target: output.targets.depth,
      present: uniform,
      lighting: output.lighting,
    });
    let (width, height) = size;
    let texture: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format));
    let (present, overlay): (&'a PresentPass, &'a OverlayPass) = (&gpu.present, &gpu.overlay);
    let builder = graph.add_raster_pass(label).parameters(&parameters);
    let builder = match &overlay_parameters {
      Some(overlay_parameters) => builder.parameters(overlay_parameters),
      None => builder,
    };

    builder
      .color(GraphColorAttachment::new(
        texture,
        wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      ))
      .record(move |context| {
        present.draw(context, format, binding, &parameters);

        if let (Some(parameters), Some(helpers)) = (&overlay_parameters, helpers) {
          overlay.draw(
            context,
            format,
            OverlayDraw {
              view: binding,
              parameters,
              overlays: None,
              helpers,
            },
          );
        }
      });

    texture
  }
}

/// A view's finished frame a capture copies out: the GPU's passes, the view, its camera and the frame it left, the
/// pass's name, the capture whose size and format it is drawn at, and its helpers' draws where they are drawn over it.
struct FinishedFrame<'a, 'v> {
  gpu: &'a GpuState,
  view: &'v SceneView,
  binding: &'a ViewBinding,
  output: &'v SceneOutput,
  label: &'static str,
  capture: &'v FrameCapture,
  helpers: Option<HelperDraws<'a>>,
}
