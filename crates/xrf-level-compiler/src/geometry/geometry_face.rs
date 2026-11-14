use xrf_math::{Vector2d, Vector3d};

/// One face of a level's static geometry as xrLC works on it (`Face`, `xrLC_Light/xrFace.h`).
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryFace {
  /// Its corners, by index into the mesh's vertices.
  pub vertices: [u32; 3],
  /// The base texture coordinate at each corner.
  pub texture_coordinates: [Vector2d; 3],
  /// Its material, by index into `build.prj`'s material table.
  pub material: u16,
  /// The game material it is made of, by its id in `gamemtl.xr`.
  pub game_material: u32,
  /// The editor's edge flags, `sm_group`: bits 0 to 2 an edge each, set where the edge is hard; bit 3 set for the back
  /// of a two-sided surface, whose edges count the other way round (`common/face_smoth_flags.h`).
  pub smoothing: u32,
  /// The face of `build.prj` it is, or was cut from.
  pub source: u32,
}

impl GeometryFace {
  /// The flag marking the back of a two-sided surface, `backface_flag`.
  pub const BACK_FACE: u32 = 1 << 3;

  /// The corners each edge runs between, `edge2idx`.
  pub const EDGES: [[usize; 2]; 3] = [[0, 1], [1, 2], [2, 0]];

  /// Whether the face is the back of a two-sided surface, `is_backface`.
  pub fn is_back_face(&self) -> bool {
    self.smoothing & Self::BACK_FACE != 0
  }

  /// Whether edge `edge` is soft, `is_soft_edge`: its bit clear, a back face's edges numbered the other way round.
  pub fn is_soft_edge(&self, edge: usize) -> bool {
    self.smoothing & (1 << self.get_edge_bit(edge)) == 0
  }

  /// Sets edge `edge` soft or hard, `set_soft_edge`.
  pub fn set_soft_edge(&mut self, edge: usize, is_soft: bool) {
    let bit: u32 = 1 << self.get_edge_bit(edge);

    if is_soft {
      self.smoothing &= !bit;
    } else {
      self.smoothing |= bit;
    }
  }

  /// Marks the face as a back face or not, `set_backface`.
  pub fn set_back_face(&mut self, is_back_face: bool) {
    if is_back_face {
      self.smoothing |= Self::BACK_FACE;
    } else {
      self.smoothing &= !Self::BACK_FACE;
    }
  }

  /// The face's corners in place.
  pub fn get_corners(&self, positions: &[Vector3d]) -> [Vector3d; 3] {
    self.vertices.map(|vertex| positions[vertex as usize])
  }

  /// The corner `vertex` is at, if the face has it, `VIndex`.
  pub fn get_corner(&self, vertex: u32) -> Option<usize> {
    self.vertices.iter().position(|corner| *corner == vertex)
  }

  /// The bit edge `edge` keeps its flag in, `convert_edge_index`: a back face's edges count the other way round.
  fn get_edge_bit(&self, edge: usize) -> usize {
    if self.is_back_face() { (4 - edge) % 3 } else { edge }
  }
}
