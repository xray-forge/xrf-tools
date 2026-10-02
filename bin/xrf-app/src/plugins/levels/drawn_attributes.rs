use xrf_visual::SectorAttributes;

/// What the viewer draws a level surface with, which is what a pack is worth carrying: the tangent frame too, whose
/// fourth bytes are the low bytes of the base coordinate, and the vertex colour water is lit by.
pub const DRAWN_ATTRIBUTES: SectorAttributes = SectorAttributes {
  binormals: true,
  colors: true,
  lightmap_uvs: true,
  normals: true,
  tangents: true,
  uvs: true,
};
