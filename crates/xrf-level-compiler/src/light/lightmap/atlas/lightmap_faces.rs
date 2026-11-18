use xrf_math::Vector2d;

/// Each face's lightmap: the page its chart sits on and its corners' coordinates there, by index into the mesh's faces;
/// none for a face no chart holds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LightmapFaces {
  pub pages: Vec<Option<u32>>,
  pub coordinates: Vec<[Vector2d; 3]>,
}

impl LightmapFaces {
  /// The textures of page `page`, `lmap#N_1` and `lmap#N_2` with `N` counted from 1 (`CLightmap::Save`).
  pub(crate) fn get_textures(page: u32) -> [String; 2] {
    [1, 2].map(|layer| format!("lmap#{}_{layer}", page + 1))
  }
}
