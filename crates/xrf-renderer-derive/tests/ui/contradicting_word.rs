use xrf_renderer_core::{GraphTexture, PassParameters};

// A second dimension would be lost to the first.
#[derive(PassParameters)]
#[parameters(group = 0)]
struct Twice {
  #[texture(d2, float, cube)]
  texture: GraphTexture,
}

fn main() {}
