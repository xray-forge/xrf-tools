#![expect(unsafe_code, reason = "creates a wgpu surface from a host's raw window handle")]

use std::cell::Cell;
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_context::GpuContext;
use crate::contract::settings::render_frame_rate::RenderFrameRate;
use crate::host::render_window_host::RenderWindowHost;
use crate::window::display_target::DisplayTarget;
use crate::window::window_backdrop::WindowBackdrop;

/// Frames the swapchain may queue ahead of the display: one fewer misses every other refresh under the window's
/// composition.
const FRAME_LATENCY: u32 = 2;

/// How many frames in a row a window may skip between the warnings that it does, so one stuck surface is reported
/// without a line every frame.
const SKIPS_REPORTED_EVERY: u32 = 600;

/// One native window's swapchain, covering its whole client area; its viewports are rectangles drawn into it.
pub struct RenderWindow {
  host: Arc<dyn RenderWindowHost>,
  surface: wgpu::Surface<'static>,
  format: wgpu::TextureFormat,
  present_modes: Vec<wgpu::PresentMode>,
  usages: wgpu::TextureUsages,
  /// What the swapchain was last configured as: its size and how it presents; none once an image asks for it to be
  /// configured again.
  configured: Cell<Option<(u32, u32, wgpu::PresentMode)>>,
  /// Whether the surface was lost, which only a new one recovers from.
  is_lost: Cell<bool>,
  /// Frames in a row the surface gave no image for, and why the last one gave none.
  skipped: Cell<(u32, &'static str)>,
  /// What its page's backdrop is painted with, once it is.
  backdrop: Option<WindowBackdrop>,
}

impl RenderWindow {
  /// # Errors
  ///
  /// Returns an error when the GPU cannot present to the window.
  pub fn new(context: &GpuContext, host: Arc<dyn RenderWindowHost>) -> XrfResult<Self> {
    // SAFETY: `RenderWindowHost`'s contract keeps the window alive until its last viewport's detach is handled, and the
    // thread lets the window's swapchain go as it handles that detach.
    let surface: wgpu::Surface<'static> = unsafe {
      context
        .instance
        .create_surface_unsafe(host.get_surface_target())
        .map_err(|error| XrfError::new_unexpected_error(format!("Cannot draw into the window: {error}")))?
    };
    let capabilities: wgpu::SurfaceCapabilities = surface.get_capabilities(&context.adapter);
    // Not sRGB: the page's colours are cleared as stated, so the area around a viewport matches the page exactly.
    let format: wgpu::TextureFormat = [wgpu::TextureFormat::Bgra8Unorm, wgpu::TextureFormat::Rgba8Unorm]
      .into_iter()
      .find(|it| capabilities.formats.contains(it))
      .or_else(|| capabilities.formats.first().copied())
      .ok_or_else(|| XrfError::new_unexpected_error("The window offers no swapchain format"))?;

    log::info!(
      "Window swapchain on {}: {format:?}, present modes {:?}",
      context.backend.get_label(),
      capabilities.present_modes
    );

    Ok(Self {
      present_modes: capabilities.present_modes,
      usages: capabilities.usages,
      configured: Cell::new(None),
      is_lost: Cell::new(false),
      skipped: Cell::new((0, "")),
      backdrop: None,
      host,
      surface,
      format,
    })
  }

  pub fn get_backdrop(&self) -> Option<&WindowBackdrop> {
    self.backdrop.as_ref()
  }

  /// Where its backdrop is kept, for the backdrop pass to make and write.
  pub fn get_backdrop_slot(&mut self) -> &mut Option<WindowBackdrop> {
    &mut self.backdrop
  }

  /// What it presents into: its swapchain, in the format it picked.
  pub fn get_target(&self) -> DisplayTarget {
    DisplayTarget { format: self.format }
  }

  /// Whether its images may be copied out of, which a capture of what the window shows needs: as far as the platform's
  /// swapchain allows.
  pub fn is_copyable(&self) -> bool {
    self.usages.contains(wgpu::TextureUsages::COPY_SRC)
  }

  /// Whether the surface was lost: the window is then drawn into through a new one.
  pub fn is_lost(&self) -> bool {
    self.is_lost.get()
  }

  /// Has the next frame configure the swapchain again, as a size change would.
  pub fn configure_again(&self) {
    self.configured.set(None);
  }

  /// Configures the swapchain for the window's size now, presenting as `frame_rate` asks, answering the size; `None`
  /// while the window shows nothing.
  pub fn ready(&mut self, context: &GpuContext, frame_rate: &RenderFrameRate) -> Option<(u32, u32)> {
    let (width, height) = self.host.get_client_size();

    if width == 0 || height == 0 || self.host.is_hidden() {
      return None;
    }

    let present_mode: wgpu::PresentMode = self.get_present_mode(frame_rate);

    if self.configured.get() != Some((width, height, present_mode)) {
      self.configure(context, width, height, present_mode);
    }

    Some((width, height))
  }

  /// How its swapchain presents at `frame_rate`: waiting for vsync, or else as soon as drawn where the surface allows.
  fn get_present_mode(&self, frame_rate: &RenderFrameRate) -> wgpu::PresentMode {
    if frame_rate.is_vsync {
      wgpu::PresentMode::Fifo
    } else {
      [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox]
        .into_iter()
        .find(|it| self.present_modes.contains(it))
        .unwrap_or(wgpu::PresentMode::Fifo)
    }
  }

  /// The next image to draw into, of the size `ready` configured; `None` where the surface gives none this time, which
  /// is logged as a run of skipped frames rather than frame by frame.
  pub fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
    let frame: Result<wgpu::SurfaceTexture, &'static str> = self.take_image();
    let (count, reason) = self.skipped.get();

    match &frame {
      Ok(_) if count > 0 => {
        log::info!("Window presents again after {count} skipped frames, the last for {reason}");
        self.skipped.set((0, ""));
      }
      Ok(_) => {}
      Err(reason) => {
        let count: u32 = count + 1;

        if count == 1 || count.is_multiple_of(SKIPS_REPORTED_EVERY) {
          log::warn!("Window frame skipped for {reason}, {count} in a row");
        }

        self.skipped.set((count, reason));
      }
    }

    frame.ok()
  }

  /// The next image, or why the surface gave none, as a log line names it.
  fn take_image(&self) -> Result<wgpu::SurfaceTexture, &'static str> {
    match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame) => Ok(frame),
      // Drawn, then configured again for what the surface has become.
      wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
        self.configured.set(None);

        Ok(frame)
      }
      // Nothing to draw into this time; the swapchain itself is fine.
      wgpu::CurrentSurfaceTexture::Timeout => Err("a timeout"),
      wgpu::CurrentSurfaceTexture::Occluded => Err("an occluded window"),
      wgpu::CurrentSurfaceTexture::Outdated => {
        self.configured.set(None);

        Err("an outdated swapchain")
      }
      // Made again through a new surface.
      wgpu::CurrentSurfaceTexture::Lost => {
        self.is_lost.set(true);

        Err("a lost surface")
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        self.configured.set(None);

        Err("a validation error")
      }
    }
  }

  fn configure(&mut self, context: &GpuContext, width: u32, height: u32, present_mode: wgpu::PresentMode) {
    self.surface.configure(
      &context.device,
      &wgpu::SurfaceConfiguration {
        // Copied out of for captures where the platform allows it.
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | (self.usages & wgpu::TextureUsages::COPY_SRC),
        format: self.format,
        width,
        height,
        present_mode,
        desired_maximum_frame_latency: FRAME_LATENCY,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        color_space: wgpu::SurfaceColorSpace::Auto,
      },
    );
    self.configured.set(Some((width, height, present_mode)));
  }
}
