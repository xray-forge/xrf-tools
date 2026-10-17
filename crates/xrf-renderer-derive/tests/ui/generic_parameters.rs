use xrf_renderer_core::{GraphTexture, PassParameters};

// Its layout is cached by name, so two instantiations would share one whatever they bind.
#[derive(PassParameters)]
#[parameters(group = 0)]
struct Generic<const SLOTS: usize> {
  #[texture(d2, float)]
  texture: GraphTexture,
}

fn main() {}
