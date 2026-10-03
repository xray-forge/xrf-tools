/// How a skinned model's vertices hang from its bones: two words a vertex, in the model's vertex order, the four
/// bones' indices as bytes, then their weights as bytes; a vertex with no weight stands as stored.
#[derive(Clone, Debug)]
pub struct StaticModelSkin {
  pub links: Vec<u32>,
  /// Bones the model's skeleton has, which a place of it reserves a matrix each for.
  pub bones: u32,
}
