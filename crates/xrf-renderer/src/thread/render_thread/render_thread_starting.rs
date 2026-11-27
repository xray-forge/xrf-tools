use std::time::{Duration, Instant};

use wgpu::util::StagingBelt;
use xrf_dlss::Dlss;

use super::RenderThread;
use crate::contract::viewport::render_rect::RenderRect;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::frame_upload::FrameUpload;
use crate::frame::timing::frame_phases::FramePhases;
use crate::thread::gpu::starting_gpu::StartingGpu;
use crate::viewport::render_viewport::RenderViewport;
use crate::viewport::reported_graph::ReportedGraph;
use crate::viewport::viewport_device::ViewportDevice;
use crate::window::display_target::DisplayTarget;
use crate::window::window_frame::WindowFrame;
use crate::window::window_viewport::WindowViewport;

impl RenderThread {
  /// Draws, while the GPU's passes are built, each window a viewport's host sent a layer for: cleared as its page's
  /// backdrop, its viewports' layers over it, presented, and counted in its viewports' reports. Every other window,
  /// and everything else of a frame, waits for the passes. Answers whether a window was presented.
  pub(super) fn draw_starting(&mut self) -> bool {
    let Some(mut starting) = self.starting.take() else {
      return false;
    };
    let started: Instant = Instant::now();
    let mut encoder: FrameEncoder = starting.begin_encoder();
    let mut windows: Vec<WindowFrame> = self.ready_layer_windows(&mut starting, &mut encoder);

    for window in &mut windows {
      if let Some(surface) = starting.surfaces.swapchains.get(&window.key) {
        window.acquire(surface);
      }
    }

    self.record_layers(&starting, &windows, &mut encoder);

    let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

    starting.context.queue.submit([commands]);
    starting.belt = Some(belt);

    let is_presented: bool = windows.iter().any(|window| window.image.is_some());
    let waited: Duration = windows.iter().map(|window| window.acquire).sum();
    let cpu: Duration = started.elapsed().saturating_sub(waited);

    self.present_windows(&starting.context.queue, windows, FramePhases::default(), cpu, None, &[]);
    self.report_starting(&mut starting);
    self.starting = Some(starting);

    is_presented
  }

  /// Readies each window a viewport's host sent a layer for: its swapchain made and configured for its size now, and
  /// its viewports' layers brought up to what was sent, their textures' changes written before anything draws them.
  fn ready_layer_windows(&mut self, starting: &mut StartingGpu, encoder: &mut FrameEncoder) -> Vec<WindowFrame> {
    let mut windows: Vec<WindowFrame> = Vec::new();
    let keys: Vec<u64> = self.hosts.keys().copied().collect();

    for key in keys {
      let is_layered: bool = self
        .viewports
        .values()
        .any(|viewport| viewport.get_window() == key && viewport.get_layer().is_sent());

      if !is_layered || !self.open_window(&mut starting.surfaces.swapchains, &starting.context, key) {
        continue;
      }

      let Some(window) = starting.surfaces.swapchains.get_mut(&key) else {
        continue;
      };
      let Some((width, height)) = window.ready(&starting.context, &self.settings.frame_rate) else {
        continue;
      };
      let target: DisplayTarget = window.get_target();
      let is_copyable: bool = window.is_copyable();

      self.prepare_layers(
        &mut starting.surfaces.layer,
        FrameUpload {
          device: &starting.context.device,
          queue: &starting.context.queue,
          encoder: &mut *encoder,
        },
        &starting.pipelines,
        key,
        target.format,
      );

      windows.push(WindowFrame {
        key,
        target,
        is_shown: true,
        width,
        height,
        is_copyable,
        // Its viewports, whose layers it draws, are counted as drawn in it; none shows a picture yet.
        drawn: self
          .viewports
          .values()
          .filter(|viewport| viewport.get_window() == key)
          .map(|viewport| WindowViewport {
            id: viewport.get_id(),
            rect: viewport.get_layout().map(|layout| layout.rect).unwrap_or_default(),
            shown: RenderRect::default(),
          })
          .collect(),
        clear: self.get_window_backdrop(key).get_clear(),
        is_backdrop_painted: false,
        image: None,
        acquire: Duration::ZERO,
      });
    }

    windows
  }

  /// Records each window that gave an image: cleared, its viewports' layers drawn over the whole of it.
  fn record_layers(&self, starting: &StartingGpu, windows: &[WindowFrame], encoder: &mut FrameEncoder) {
    for window in windows {
      let Some(image) = &window.image else {
        continue;
      };
      let view: wgpu::TextureView = image.texture.create_view(&Default::default());
      let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("window layers"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(window.clear),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      let layers = self
        .viewports
        .values()
        .filter(|viewport| viewport.get_window() == window.key)
        .map(RenderViewport::get_layer)
        .filter(|layer| layer.is_drawn());

      for layer in layers {
        starting
          .surfaces
          .layer
          .draw(&mut pass, window.target.format, window.width, window.height, layer);
      }
    }
  }

  /// Reports each viewport whose report is due: its frames, and the device as far as it started; DLSS is told as none
  /// until its start ended, which the frame does not wait for.
  fn report_starting(&mut self, starting: &mut StartingGpu) {
    let now: Instant = Instant::now();
    let dlss: Option<&Dlss> = if starting.context.dlss.is_started() {
      starting.context.dlss.get()
    } else {
      None
    };
    let device: ViewportDevice<'_> = ViewportDevice::of(&starting.context, dlss);

    for viewport in self.viewports.values_mut() {
      viewport.report(
        now,
        device,
        None,
        &mut self.scenes,
        ReportedGraph {
          timer: &mut starting.runtime.timer,
          composition: &[],
          graph: None,
        },
      );
    }
  }
}
