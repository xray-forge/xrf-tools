use xrf_geometry::{ProgressiveMesh, ProgressivePoint, ProgressiveWindow};
use xrf_math::{EPS_L, Vector2d, Vector3d};

use crate::geometry::visual::geometry_fast_path::GeometryFastPath;
use crate::geometry::visual::geometry_visual_light::GeometryVisualLight;
use crate::geometry::visual::geometry_visual_vertex::GeometryVisualVertex;

/// One static visual as xrLC builds it from a split (`Flex2OGF`, `xrLC/xrFlex2OGF.cpp`): its dress, its welded
/// vertices and faces, and the fast path's positions for an opaque surface.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryVisual {
  /// The `build.prj` material of its faces.
  pub material: u16,
  pub sector: u16,
  /// Its textures as the shader table names them: the base, then `lmap#N_1` and `lmap#N_2`, or the base's `_lm`.
  pub textures: Vec<String>,
  /// Its shader table entry, `<render shader>/<texture>,<texture>...` (`OGF::Save`).
  pub shader: String,
  pub(crate) light: GeometryVisualLight,
  pub vertices: Vec<GeometryVisualVertex>,
  pub faces: Vec<[u16; 3]>,
  /// The fast path's positions and faces (`fast_path_data`), empty for a surface with alpha.
  pub fast_vertices: Vec<Vector3d>,
  pub fast_faces: Vec<[u16; 3]>,
  /// A progressive visual's windows over its faces, and over its fast path's where it has one; none for a static one.
  pub windows: Option<Vec<ProgressiveWindow>>,
  pub(crate) fast_windows: Option<Vec<ProgressiveWindow>>,
}

impl GeometryVisual {
  /// Fewer faces than this are not worth making progressive (`c_PM_FaceLimit`).
  pub(crate) const PROGRESSIVE_FACE_LIMIT: usize = 128;

  /// The share of a static visual's vertices its windows must step through for it to be made progressive
  /// (`c_PM_MetricLimit_static`).
  pub(crate) const STATIC_METRIC_LIMIT: f32 = 0.10;

  /// The same for a multi-use model's visual (`c_PM_MetricLimit_mu`).
  pub(crate) const MODEL_METRIC_LIMIT: f32 = 0.05;

  /// A visual of no geometry yet.
  pub(crate) fn new(
    material: u16,
    sector: u16,
    textures: Vec<String>,
    shader: String,
    light: GeometryVisualLight,
  ) -> Self {
    Self {
      material,
      sector,
      textures,
      shader,
      light,
      vertices: Vec::new(),
      faces: Vec::new(),
      fast_vertices: Vec::new(),
      fast_faces: Vec::new(),
      windows: None,
      fast_windows: None,
    }
  }

  /// `OGF::MakeProgressive`: a visual of [`Self::PROGRESSIVE_FACE_LIMIT`] faces or more made progressive
  /// ([`ProgressiveMesh`]) over its positions and base coordinates, kept so where its windows step through at least
  /// `metric_limit` of its vertices, counted as xrLC counts them, the collapses and the whole mesh's window. Its fast
  /// path is then made progressive over positions alone, with no such limit; where that fails, the visual stays as it
  /// was.
  pub(crate) fn make_progressive(&mut self, metric_limit: f32) {
    if self.faces.len() < Self::PROGRESSIVE_FACE_LIMIT {
      return;
    }

    let points: Vec<ProgressivePoint> = self
      .vertices
      .iter()
      .map(|vertex| ProgressivePoint {
        position: vertex.position,
        coordinate: vertex.texture_coordinate,
      })
      .collect();
    let Some(mesh) = ProgressiveMesh::build(&points, &self.faces) else {
      return;
    };

    if (mesh.windows.len() as f32 / self.vertices.len() as f32) < metric_limit {
      return;
    }

    let fast: Option<ProgressiveMesh> = if self.fast_vertices.is_empty() || self.fast_faces.is_empty() {
      None
    } else {
      let points: Vec<ProgressivePoint> = self
        .fast_vertices
        .iter()
        .map(|position| ProgressivePoint {
          position: *position,
          coordinate: Vector2d::new(0.0, 0.0),
        })
        .collect();

      match ProgressiveMesh::build(&points, &self.fast_faces) {
        Some(fast) => Some(fast),
        None => return,
      }
    };

    self.vertices = Self::permute(std::mem::take(&mut self.vertices), &mesh.permutation);
    self.faces = mesh.faces;
    self.windows = Some(mesh.windows);

    if let Some(fast) = fast {
      self.fast_vertices = Self::permute(std::mem::take(&mut self.fast_vertices), &fast.permutation);
      self.fast_faces = fast.faces;
      self.fast_windows = Some(fast.windows);
    }
  }

  /// Its triangles at full detail: a progressive visual's index buffer holds every level's run.
  pub fn get_triangle_count(&self) -> usize {
    self
      .windows
      .as_ref()
      .and_then(|windows| windows.first())
      .map_or(self.faces.len(), |window| usize::from(window.triangles))
  }

  /// `vertices` each moved to the place `permutation` gives it.
  fn permute<V: Copy>(vertices: Vec<V>, permutation: &[u16]) -> Vec<V> {
    let mut placed: Vec<Option<V>> = vec![None; vertices.len()];

    for (vertex, place) in vertices.into_iter().zip(permutation) {
      placed[usize::from(*place)] = Some(vertex);
    }

    placed.into_iter().flatten().collect()
  }

  /// `OGF::_BuildFace`: each corner welded to the first vertex similar to it ([`Self::is_similar`]) or added; a face
  /// closing on itself drops the vertices it added, a face repeating one already held, in any order of its corners,
  /// is left out with its vertices kept; a face taken is added to the fast path too as `fast_path` says.
  ///
  /// `texel` is a texel of each coordinate channel's texture, `1 / width` and `1 / height`.
  pub(crate) fn add_face(
    &mut self,
    corners: [GeometryVisualVertex; 3],
    texel: &[Vector2d],
    fast_path: GeometryFastPath,
  ) {
    let held: usize = self.vertices.len();
    let face: [u16; 3] = corners.map(|corner| self.weld(corner, texel));

    if Self::is_degenerate(&face) {
      self.vertices.truncate(held);
      return;
    }

    if self.faces.iter().any(|other| Self::is_same(other, &face)) {
      return;
    }

    self.faces.push(face);

    if fast_path == GeometryFastPath::Joined {
      self.add_fast_face(corners.map(|corner| corner.position));
    }
  }

  /// `OGF::x_BuildFace`: the corners' positions welded within `EPS_L`, a face closing on itself dropping the vertices
  /// it added.
  fn add_fast_face(&mut self, corners: [Vector3d; 3]) {
    let held: usize = self.fast_vertices.len();
    let face: [u16; 3] = corners.map(|corner| {
      match self
        .fast_vertices
        .iter()
        .position(|vertex| Self::is_near(vertex, &corner))
      {
        Some(index) => index as u16,
        None => {
          self.fast_vertices.push(corner);
          (self.fast_vertices.len() - 1) as u16
        }
      }
    });

    if Self::is_degenerate(&face) {
      self.fast_vertices.truncate(held);
    } else {
      self.fast_faces.push(face);
    }
  }

  /// `OGF::_BuildVertex`.
  fn weld(&mut self, corner: GeometryVisualVertex, texel: &[Vector2d]) -> u16 {
    match self
      .vertices
      .iter()
      .position(|vertex| Self::is_similar(vertex, &corner, texel))
    {
      Some(index) => index as u16,
      None => {
        self.vertices.push(corner);
        (self.vertices.len() - 1) as u16
      }
    }
  }

  /// `OGF_Vertex::similar`: position and normal within `EPS_L`, the basis within a step, each coordinate within a texel
  /// of its channel's texture, all strictly.
  fn is_similar(first: &GeometryVisualVertex, second: &GeometryVisualVertex, texel: &[Vector2d]) -> bool {
    let near = |one: &Vector2d, other: &Vector2d, texel: &Vector2d| {
      (one.x - other.x).abs() < texel.x && (one.y - other.y).abs() < texel.y
    };

    Self::is_near(&first.position, &second.position)
      && Self::is_near(&first.normal, &second.normal)
      && first.basis.is_similar(&second.basis)
      && near(&first.texture_coordinate, &second.texture_coordinate, &texel[0])
      && match (first.lightmap_coordinate, second.lightmap_coordinate) {
        (Some(one), Some(other)) => near(&one, &other, &texel[1]),
        _ => true,
      }
  }

  /// `Fvector::similar` at its default: every component strictly within `EPS_L`.
  fn is_near(first: &Vector3d, second: &Vector3d) -> bool {
    (first.x - second.x).abs() < EPS_L && (first.y - second.y).abs() < EPS_L && (first.z - second.z).abs() < EPS_L
  }

  /// `OGF_Face::Degenerate`.
  fn is_degenerate([first, second, third]: &[u16; 3]) -> bool {
    first == second || first == third || second == third
  }

  /// `OGF_Face::Equal`: the same three vertices in any order.
  fn is_same(first: &[u16; 3], second: &[u16; 3]) -> bool {
    let mut one: [u16; 3] = *first;
    let mut other: [u16; 3] = *second;

    one.sort_unstable();
    other.sort_unstable();
    one == other
  }

  /// `OGF::Optimize` where the compiler shader optimizes coordinates: the faces gathered into groups joined by shared
  /// vertices, each sweep in face order taking any face sharing a vertex with the group, until a sweep adds no vertex;
  /// then each group's base coordinates moved by the whole tiles nearest the middle of their span, whose upper end
  /// starts from the smallest positive float. A vertex an earlier group took is not taken again.
  pub(crate) fn shift_texture_coordinates(&mut self) {
    let mut is_vertex_taken: Vec<bool> = vec![false; self.vertices.len()];
    let mut is_face_taken: Vec<bool> = vec![false; self.faces.len()];

    loop {
      let mut group: Vec<u16> = Vec::new();

      loop {
        let before: usize = group.len();

        self.select_adjacent(&mut group, &mut is_vertex_taken, &mut is_face_taken);

        if group.len() == before {
          break;
        }
      }

      if group.is_empty() {
        break;
      }

      let mut low: Vector2d = Vector2d::new(f32::MAX, f32::MAX);
      let mut high: Vector2d = Vector2d::new(f32::MIN_POSITIVE, f32::MIN_POSITIVE);

      for vertex in &group {
        let coordinate: Vector2d = self.vertices[usize::from(*vertex)].texture_coordinate;

        low = Vector2d::new(low.x.min(coordinate.x), low.y.min(coordinate.y));
        high = Vector2d::new(high.x.max(coordinate.x), high.y.max(coordinate.y));
      }

      let shift: Vector2d = Vector2d::new(
        ((high.x - low.x) / 2.0 + low.x).floor(),
        ((high.y - low.y) / 2.0 + low.y).floor(),
      );

      for vertex in &group {
        let coordinate: &mut Vector2d = &mut self.vertices[usize::from(*vertex)].texture_coordinate;

        *coordinate = Vector2d::new(coordinate.x - shift.x, coordinate.y - shift.y);
      }
    }
  }

  /// `OGF::adjacent_select`: one sweep over the faces not yet taken.
  fn select_adjacent(&self, group: &mut Vec<u16>, is_vertex_taken: &mut [bool], is_face_taken: &mut [bool]) {
    for (index, face) in self.faces.iter().enumerate() {
      if is_face_taken[index] {
        continue;
      }

      if !group.is_empty() && !face.iter().any(|vertex| group.contains(vertex)) {
        continue;
      }

      let is_first: bool = group.is_empty();

      is_face_taken[index] = true;

      for vertex in face {
        if is_first || !is_vertex_taken[usize::from(*vertex)] {
          group.push(*vertex);
          is_vertex_taken[usize::from(*vertex)] = true;
        }
      }
    }
  }
}
