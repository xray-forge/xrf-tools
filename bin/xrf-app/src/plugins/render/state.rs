use xrf_renderer::Renderer;

/// The application's one renderer, which starts a GPU only once a viewport is attached.
#[derive(Default)]
pub struct RenderState {
  pub renderer: Renderer,
}
