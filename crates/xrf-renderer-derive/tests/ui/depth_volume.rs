use xrf_renderer_core::{GraphTexture, PassParameters};

// A depth texture of three dimensions, which WGSL has no type for.
#[derive(PassParameters)]
#[parameters(group = 0)]
struct DepthVolume {
  #[binding(0)]
  #[texture(d3, depth)]
  depth: GraphTexture,
}

fn main() {}
