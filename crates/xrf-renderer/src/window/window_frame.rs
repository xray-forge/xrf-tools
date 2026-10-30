use std::time::{Duration, Instant};

use xrf_renderer_core::{FrameGraph, GraphBindings, GraphColorAttachment, GraphTexture, GraphTextureAccess};

use crate::contract::render_rect::RenderRect;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::pass::backdrop_pass::BackdropPass;
use crate::pass::grid_pass::GridPass;
use crate::pass::overlay_pass::OverlayPass;
use crate::pass::present_pass::PresentPass;
use crate::pass::view_binding::ViewBinding;
use crate::window::composed_view::ComposedView;
use crate::window::render_window::RenderWindow;
use crate::window::window_backdrop::WindowBackdrop;

/// A window this frame draws: its format, each viewport drawn into it with its whole rectangle and the part the window
/// shows, what the page is cleared to around them and whether its backdrop is painted over that; and the image it draws
/// into, taken only once every view's work is recorded, so the render thread readies the frame while the GPU still draws
/// the last.
pub struct WindowFrame {
  pub key: u64,
  pub format: wgpu::TextureFormat,
  /// Whether it shows anything this frame: one minimised or sized to nothing draws only its viewports' captures, which
  /// draw into textures of their own, and takes no image.
  pub is_shown: bool,
  pub drawn: Vec<(RenderViewportId, RenderRect, RenderRect)>,
  pub clear: wgpu::Color,
  pub is_backdrop_painted: bool,
  /// The image, once acquired; none before, or where the surface gave none.
  pub image: Option<wgpu::SurfaceTexture>,
  /// How long the image took to acquire.
  pub acquire: Duration,
}

impl WindowFrame {
  /// Takes the window's next image, timing the wait.
  pub fn acquire(&mut self, window: &RenderWindow) {
    let acquiring: Instant = Instant::now();

    self.image = {
      profiling::scope!("acquire");

      window.acquire()
    };
    self.acquire = acquiring.elapsed();
  }

  /// Declares the window's composition into `target`: the page's backdrop, each viewport's picture or its grid cropped
  /// to what the window shows, its overlays, then the captures copied out of it.
  pub fn declare<'a>(
    &self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (target, backdrop): (&'a wgpu::TextureView, Option<&'a WindowBackdrop>),
    composed: Vec<ComposedView<'a>>,
    (backdrop_pass, present, overlay, grid): (&'a BackdropPass, &'a PresentPass, &'a OverlayPass, &'a GridPass),
    captures: &'a [(RenderViewportId, FrameCapture, CaptureReply)],
  ) {
    let format: wgpu::TextureFormat = self.format;

    graph.begin_group("window");

    let window_texture: GraphTexture = bindings.import_view(graph, "window", target);
    let declared = composed.iter().fold(
      graph.add_raster_pass("window").color(GraphColorAttachment::new(
        window_texture,
        wgpu::LoadOp::Clear(self.clear),
      )),
      |builder, view| {
        let builder = match &view.present {
          Some(present) => builder.parameters(present),
          None => builder,
        };

        match &view.overlays {
          Some((overlay, _)) => builder.parameters(overlay),
          None => builder,
        }
      },
    );

    declared.record(move |context| {
      if let Some(backdrop) = backdrop {
        backdrop_pass.draw(context.get_pass(), format, backdrop);
      }

      for view in &composed {
        let (rect, shown, binding): (RenderRect, RenderRect, &ViewBinding) = (view.rect, view.shown, view.binding);
        let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

        // The whole viewport, past the window's edges where it reaches them, cropped to what the window shows.
        pass.set_viewport(
          rect.x as f32,
          rect.y as f32,
          rect.width as f32,
          rect.height as f32,
          0.0,
          1.0,
        );
        pass.set_scissor_rect(shown.x as u32, shown.y as u32, shown.width, shown.height);

        match &view.present {
          Some(parameters) => present.draw(context, format, binding, parameters),
          None => grid.draw(pass, binding),
        }

        if let Some((parameters, overlays)) = &view.overlays {
          overlay.draw(context, format, (binding, parameters), (overlays, view.helpers));
        }
      }
    });

    for (id, capture, _) in captures {
      if capture.is_screenshot() || !self.drawn.iter().any(|(drawn, ..)| drawn == id) {
        continue;
      }

      graph
        .add_encoder_pass("capture")
        .texture(window_texture, GraphTextureAccess::CopySource)
        .keep()
        .record(move |context| {
          let texture: &wgpu::Texture = context.get_texture(window_texture).texture;

          capture.encode(context.get_encoder(), texture);
        });
    }
  }
}
