use crate::host::audio_material::AudioMaterial;

/// Triangles and their materials, in the game's left-handed space: `x` right, `y` up, `z` ahead.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioSceneMesh {
  pub vertices: Vec<[f32; 3]>,
  /// Each triangle's corners, by index into [`Self::vertices`], in the game's winding.
  pub triangles: Vec<[u32; 3]>,
  /// Each triangle's material, by index into [`Self::materials`].
  pub material_indices: Vec<u32>,
  pub materials: Vec<AudioMaterial>,
}

impl AudioSceneMesh {
  /// Whether every triangle names vertices and a material the mesh holds.
  pub fn is_valid(&self) -> bool {
    let vertices: usize = self.vertices.len();

    self.triangles.len() == self.material_indices.len()
      && self
        .triangles
        .iter()
        .all(|triangle| triangle.iter().all(|index| (*index as usize) < vertices))
      && self
        .material_indices
        .iter()
        .all(|index| (*index as usize) < self.materials.len())
  }
}
