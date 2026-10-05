use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_context::GpuContext;
use crate::host::render_window_host::RenderWindowHost;
use crate::window::window_backdrop::WindowBackdrop;

/// Frames the swapchain may queue ahead of the display: one fewer misses every other refresh under the window's
/// composition.
const FRAME_LATENCY: u32 = 2;

/// One native window's swapchain, covering its whole client area; its viewports are rectangles drawn into it.
pub struct RenderWindow {
  host: Arc<dyn RenderWindowHost>,
  surface: wgpu::Surface<'static>,
  format: wgpu::TextureFormat,
  present_modes: Vec<wgpu::PresentMode>,
  usages: wgpu::TextureUsages,
  /// What the swapchain was last configured as: its size and whether it waits for vsync.
  configured: Option<(u32, u32, bool)>,
  /// Whether the surface was lost, which only a new one recovers from.
  is_lost: bool,
  /// What its page's backdrop is painted with, once it is.
  backdrop: Option<WindowBackdrop>,
}

impl RenderWindow {
  /// # Errors
  ///
  /// Returns an error when the GPU cannot present to the window.
  pub fn new(context: &GpuContext, host: Arc<dyn RenderWindowHost>) -> XrfResult<Self> {
    let (window, display) = host.get_handles();
    // Safety: the host keeps the window alive for as long as it hosts viewports, and a window's swapchain goes as
    // its last viewport detaches.
    let surface: wgpu::Surface<'static> = unsafe {
      context
        .instance
        .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
          raw_display_handle: Some(display),
          raw_window_handle: window,
        })
        .map_err(|error| XrfError::new_unexpected_error(format!("Cannot draw into the window: {error}")))?
    };
    let capabilities: wgpu::SurfaceCapabilities = surface.get_capabilities(&context.adapter);
    // Not sRGB: the page's colours are cleared as stated, so the area around a viewport matches the page exactly.
    let format: wgpu::TextureFormat = [wgpu::TextureFormat::Bgra8Unorm, wgpu::TextureFormat::Rgba8Unorm]
      .into_iter()
      .find(|it| capabilities.formats.contains(it))
      .or_else(|| capabilities.formats.first().copied())
      .ok_or_else(|| XrfError::new_unexpected_error("The window offers no swapchain format"))?;

    Ok(Self {
      present_modes: capabilities.present_modes,
      usages: capabilities.usages,
      configured: None,
      is_lost: false,
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

  pub fn get_format(&self) -> wgpu::TextureFormat {
    self.format
  }

  /// Whether the surface was lost: the window is then drawn into through a new one.
  pub fn is_lost(&self) -> bool {
    self.is_lost
  }

  /// The next frame to draw, configured for the window's size now, or `None` while the window shows nothing.
  pub fn acquire(&mut self, context: &GpuContext, is_vsync: bool) -> Option<(wgpu::SurfaceTexture, u32, u32)> {
    let (width, height) = self.host.get_client_size();

    if width == 0 || height == 0 || self.host.is_minimized() {
      return None;
    }

    if self.configured != Some((width, height, is_vsync)) {
      self.configure(context, width, height, is_vsync);
    }

    match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame) => Some((frame, width, height)),
      // Drawn, then configured again for what the surface has become.
      wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
        self.configured = None;

        Some((frame, width, height))
      }
      // Nothing to draw into this time; the swapchain itself is fine.
      wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => None,
      wgpu::CurrentSurfaceTexture::Outdated => {
        self.configured = None;

        None
      }
      wgpu::CurrentSurfaceTexture::Lost => {
        log::warn!("Window surface lost, recreating it");
        self.is_lost = true;

        None
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        log::warn!("Window frame skipped on a validation error");
        self.configured = None;

        None
      }
    }
  }

  fn configure(&mut self, context: &GpuContext, width: u32, height: u32, is_vsync: bool) {
    let present_mode: wgpu::PresentMode = if is_vsync {
      wgpu::PresentMode::Fifo
    } else {
      [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox]
        .into_iter()
        .find(|it| self.present_modes.contains(it))
        .unwrap_or(wgpu::PresentMode::Fifo)
    };

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
    self.configured = Some((width, height, is_vsync));
  }
}
