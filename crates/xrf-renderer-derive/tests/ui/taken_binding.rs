use xrf_renderer_core::{GraphTexture, PassParameters};

// The second field takes the index the first already has.
#[derive(PassParameters)]
#[parameters(group = 0)]
struct Twice {
  #[binding(2)]
  #[texture(d2, float)]
  first: GraphTexture,
  #[binding(2)]
  #[texture(d2, float)]
  second: GraphTexture,
}

fn main() {}
