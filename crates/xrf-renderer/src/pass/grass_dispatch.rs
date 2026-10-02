/// What one frame's grass planting dispatches over: the ring's cells, its bands, the models, and the items the lists
/// hold.
#[derive(Clone, Copy, Debug)]
pub struct GrassDispatch {
  pub cells: u32,
  pub bands: u32,
  pub models: u32,
  pub capacity: u32,
}
