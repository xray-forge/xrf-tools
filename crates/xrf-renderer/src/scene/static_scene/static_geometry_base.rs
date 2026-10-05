/// Where one geometry's vertices and indices start in what an item writes, in its own layout's words and its indices.
#[derive(Clone, Copy, Debug)]
pub struct StaticGeometryBase {
  pub vertex_start: u32,
  pub index_start: u32,
}
