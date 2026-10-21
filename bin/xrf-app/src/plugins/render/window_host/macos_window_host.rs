use std::ffi::c_void;
use std::num::NonZeroIsize;

use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{ClassType, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSAutoresizingMaskOptions, NSView, NSWindow, NSWindowOcclusionState, NSWindowOrderingMode};
use objc2_quartz_core::{CALayer, CAMetalLayer};
use xrf_renderer::{RenderWindowHost, SurfaceTargetUnsafe};

define_class!(
  /// The view under the webview that viewports are drawn in, backed by a Metal layer.
  ///
  /// Backed rather than hosting one, so AppKit keeps the layer's bounds in step with the view. Its scale it leaves at 1
  /// for a layer the view makes, so the view gives it the window's backing scale whenever that changes, as `MTKView`
  /// does: the swapchain is sized in the device pixels the page measures viewports in.
  // SAFETY: `NSView` asks nothing of a subclass that overrides `makeBackingLayer` and `viewDidChangeBackingProperties`,
  // calling the latter's own first, and this one has no `Drop`.
  #[unsafe(super(NSView))]
  #[thread_kind = MainThreadOnly]
  #[name = "XrfRendererView"]
  struct RendererView;

  impl RendererView {
    #[unsafe(method_id(makeBackingLayer))]
    fn make_backing_layer(&self) -> Retained<CALayer> {
      Retained::into_super(CAMetalLayer::new())
    }

    #[unsafe(method(viewDidChangeBackingProperties))]
    fn view_did_change_backing_properties(&self) {
      // SAFETY: `NSView`'s own implementation, which takes nothing and answers nothing.
      let _: () = unsafe { msg_send![super(self), viewDidChangeBackingProperties] };

      self.follow_backing_scale();
    }
  }
);

impl RendererView {
  /// Gives the layer the backing scale of the window the view is in, none while it is in none.
  fn follow_backing_scale(&self) {
    if let (Some(layer), Some(window)) = (self.layer(), self.window()) {
      layer.setContentsScale(window.backingScaleFactor());
    }
  }
}

/// A macOS window viewports are drawn into, through the Metal layer of a view of its own under the webview.
///
/// The webview shows the view through the pixels its page leaves transparent, as WebView2 shows the swapchain on Windows.
/// wgpu makes a Metal surface from a view on the main thread only, and from a layer on any, so the view is made and
/// placed when a page attaches and the render thread is handed its layer.
pub struct MacosWindowHost {
  window: NonZeroIsize,
  view: NonZeroIsize,
  layer: NonZeroIsize,
}

impl MacosWindowHost {
  /// The host of the `NSWindow` at `window`, its renderer view placed under the webview by the first attach and found by
  /// every later one.
  ///
  /// # Errors
  ///
  /// Returns an error off the main thread, where AppKit cannot be touched (Tauri runs a synchronous command on it), and
  /// for a window with no content view.
  pub fn attach(window: NonZeroIsize) -> Result<Self, String> {
    let marker: MainThreadMarker =
      MainThreadMarker::new().ok_or_else(|| "A native viewport attaches on the main thread only".to_string())?;
    // SAFETY: the handle is the `NSWindow` the window registered when built and forgets when destroyed, read on the main
    // thread, which alone destroys it.
    let ns_window: &NSWindow = unsafe { &*(window.get() as *const NSWindow) };
    let content: Retained<NSView> = ns_window
      .contentView()
      .ok_or_else(|| "The window has no content view to draw under".to_string())?;
    let view: Retained<NSView> = Self::find_view(&content).unwrap_or_else(|| Self::insert_view(&content, marker));
    let layer: Retained<CALayer> = view
      .layer()
      .ok_or_else(|| "The renderer's view has no layer".to_string())?;

    Ok(Self {
      window,
      view: Self::to_handle(Retained::as_ptr(&view).cast()),
      // The view keeps it while it is in the window; its class makes every backing layer a Metal one.
      layer: Self::to_handle(Retained::as_ptr(&layer).cast()),
    })
  }

  fn find_view(content: &NSView) -> Option<Retained<NSView>> {
    content
      .subviews()
      .iter()
      .find(|view| view.isKindOfClass(RendererView::class()))
  }

  fn insert_view(content: &NSView, marker: MainThreadMarker) -> Retained<NSView> {
    // SAFETY: `initWithFrame:` is `NSView`'s designated initialiser, which the subclass inherits as it is.
    let view: Retained<RendererView> =
      unsafe { msg_send![RendererView::alloc(marker), initWithFrame: content.bounds()] };

    view.setWantsLayer(true);
    view
      .setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable);
    content.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Below, None);
    // The first placement need not count as a change of backing properties, so the scale is given now as well.
    view.follow_backing_scale();

    Retained::into_super(view)
  }

  fn to_handle(pointer: *const c_void) -> NonZeroIsize {
    NonZeroIsize::new(pointer as isize).expect("A retained object is never at null")
  }

  fn get_layer(&self) -> &CAMetalLayer {
    // SAFETY: the view holds the layer while it is in the window, which `RenderWindowHost`'s contract keeps alive.
    unsafe { &*(self.layer.get() as *const CAMetalLayer) }
  }
}

// SAFETY: the view is the window's for as long as the window lives and is never removed from it, and a window whose
// close its page approved is held open until the render thread handled its viewports' detach, as on Windows
// (`RenderState::close_window`).
unsafe impl RenderWindowHost for MacosWindowHost {
  fn get_key(&self) -> u64 {
    self.view.get() as u64
  }

  fn get_surface_target(&self) -> SurfaceTargetUnsafe {
    SurfaceTargetUnsafe::CoreAnimationLayer(self.layer.get() as *mut c_void)
  }

  fn get_client_size(&self) -> (u32, u32) {
    // Core Animation lets any thread read a layer's geometry.
    let layer: &CAMetalLayer = self.get_layer();
    let size = layer.bounds().size;
    let scale: f64 = layer.contentsScale();

    (
      (size.width * scale).round().max(0.0) as u32,
      (size.height * scale).round().max(0.0) as u32,
    )
  }

  fn is_hidden(&self) -> bool {
    // SAFETY: the window outlives this host, as `RenderWindowHost`'s contract keeps it. AppKit documents its state for
    // the main thread; the occlusion bits are read off it here as wgpu's Metal surface reads them before acquiring each
    // drawable on the same thread (`wgpu-hal/src/metal/surface.rs`), so this read adds no access it does not make.
    let window: &NSWindow = unsafe { &*(self.window.get() as *const NSWindow) };

    !window.occlusionState().contains(NSWindowOcclusionState::Visible)
  }
}
