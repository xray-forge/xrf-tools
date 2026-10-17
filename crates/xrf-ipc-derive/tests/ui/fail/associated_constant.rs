// A constant of a type has no module to register from: Rust allows no module in an `impl`.
struct Shadows;

impl Shadows {
  #[xrf_ipc::constant]
  const MAX_CASCADES: u32 = 4;
}

fn main() {}
