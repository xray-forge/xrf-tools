use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_context::GpuContext;
use crate::contract::render_presentation::RenderPresentation;
use crate::host::render_window_host::RenderWindowHost;

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
  /// What the swapchain was last configured as: its size and presentation.
  configured: Option<(u32, u32, RenderPresentation)>,
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
      host,
      surface,
      format,
    })
  }

  pub fn get_host(&self) -> &Arc<dyn RenderWindowHost> {
    &self.host
  }

  pub fn get_format(&self) -> wgpu::TextureFormat {
    self.format
  }

  /// The next frame to draw, configured for the window's size now, or `None` while the window shows nothing.
  pub fn acquire(
    &mut self,
    context: &GpuContext,
    presentation: RenderPresentation,
  ) -> Option<(wgpu::SurfaceTexture, u32, u32)> {
    let (width, height) = self.host.get_client_size();

    if width == 0 || height == 0 || self.host.is_minimized() {
      return None;
    }

    if self.configured != Some((width, height, presentation)) {
      self.configure(context, width, height, presentation);
    }

    match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
        Some((frame, width, height))
      }
      other => {
        log::warn!("Window frame skipped: {other:?}");
        self.configured = None;

        None
      }
    }
  }

  fn configure(&mut self, context: &GpuContext, width: u32, height: u32, presentation: RenderPresentation) {
    let present_mode: wgpu::PresentMode = match presentation {
      RenderPresentation::Vsync => wgpu::PresentMode::Fifo,
      RenderPresentation::Uncapped => [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox]
        .into_iter()
        .find(|it| self.present_modes.contains(it))
        .unwrap_or(wgpu::PresentMode::Fifo),
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
    self.configured = Some((width, height, presentation));
  }
}
