use xrf_renderer_core::{GraphTexture, PassParameters};

// A second group would replace the first.
#[derive(PassParameters)]
#[parameters(group = 0, group = 1)]
struct Regrouped {
  #[texture(d2, float)]
  texture: GraphTexture,
}

fn main() {}
