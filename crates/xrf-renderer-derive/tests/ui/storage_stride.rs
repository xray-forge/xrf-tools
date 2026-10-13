use glam::Vec3;
use xrf_renderer_core::{PassParameters, StorageArray};

// WGSL places a `vec3` array's elements sixteen bytes apart; Rust packs them twelve apart.
#[derive(PassParameters)]
#[parameters(group = 0)]
struct Points {
  #[storage]
  points: StorageArray<Vec3>,
}

fn main() {}
