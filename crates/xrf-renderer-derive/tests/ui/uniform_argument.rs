use xrf_renderer_core::{GraphBuffer, PassParameters};

// A uniform takes no arguments, which would be dropped unread.
#[derive(PassParameters)]
#[parameters(group = 0)]
struct Dynamic {
  #[uniform(dynamic)]
  buffer: GraphBuffer,
}

fn main() {}
