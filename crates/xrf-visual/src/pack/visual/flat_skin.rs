/// One submesh's skinning links, flattened four per vertex and paired so neither can be pushed without the other.
pub(crate) struct FlatSkin {
  pub(crate) indices: Vec<u16>,
  pub(crate) weights: Vec<f32>,
}
