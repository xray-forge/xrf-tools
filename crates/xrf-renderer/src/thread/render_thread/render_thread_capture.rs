use std::collections::BTreeMap;

use xrf_error::XrfError;
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureAccess, GraphTextureDescriptor,
};

use crate::capture::capture_job::CaptureJob;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::pass::present_parameters::PresentParameters;
use crate::pass::present_pass::PresentPass;
use crate::scene::level::scene_output::SceneOutput;
use crate::thread::gpu_state::GpuState;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::window_frame::WindowFrame;

use super::{RenderThread, ShotCopy};

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
          let reply: CaptureReply = viewport.screenshots.remove(0);

          match FrameCapture::new_screenshot(device, window.format, (rect.width, rect.height), viewport.frame) {
            Ok(capture) => captures.push((*id, capture, reply)),
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
  /// its window shows it but without its overlays or its selection's outline, then copied out.
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
    let (width, height): (u32, u32) = capture.get_size();
    let format: wgpu::TextureFormat = capture.get_format();
    let parameters: PresentParameters<'a> = PresentParameters {
      present: runtime.push_uniform(&view.get_present_uniform().for_screenshot((width, height))),
      ..Self::to_present_parameters(gpu, output)
    };
    let texture: GraphTexture =
      graph.create_texture(GraphTextureDescriptor::new_2d("screenshot", width, height, format));
    let present: &'a PresentPass = &gpu.present;

    graph
      .add_raster_pass("screenshot")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(
        texture,
        wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      ))
      .record(move |context| present.draw(context, format, binding, &parameters));
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
    let (width, height): (u32, u32) = capture.get_size();
    let format: wgpu::TextureFormat = capture.get_format();
    let parameters: PresentParameters<'a> = PresentParameters {
      present: runtime.push_uniform(&view.get_present_uniform().for_screenshot((width, height))),
      ..Self::to_present_parameters(gpu, output)
    };
    let texture: GraphTexture =
      graph.create_texture(GraphTextureDescriptor::new_2d("capture shot", width, height, format));
    let depth: GraphTexture = output.targets.depth;
    let present: &'a PresentPass = &gpu.present;

    graph
      .add_raster_pass("capture shot")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(
        texture,
        wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      ))
      .record(move |context| present.draw(context, format, binding, &parameters));
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
}
