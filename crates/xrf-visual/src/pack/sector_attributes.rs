use xrf_level::LevelVertexLayout;

/// Which attributes a sector's vertices carry, as the declarations of its ranges together declare them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SectorAttributes {
  pub normals: bool,
  pub tangents: bool,
  pub binormals: bool,
  pub uvs: bool,
  pub lightmap_uvs: bool,
  pub colors: bool,
}

impl SectorAttributes {
  /// Widens what the sector carries by what one more declaration does.
  pub fn widen(&mut self, layout: &LevelVertexLayout) {
    self.binormals |= layout.get_binormal_offset().is_some();
    self.colors |= layout.get_color_offset().is_some();
    self.lightmap_uvs |= layout.get_lightmap_coordinate_offset().is_some();
    self.normals |= layout.get_normal_offset().is_some();
    self.tangents |= layout.get_tangent_offset().is_some();
    self.uvs |= layout.get_texture_coordinate_offset().is_some();
  }
}
