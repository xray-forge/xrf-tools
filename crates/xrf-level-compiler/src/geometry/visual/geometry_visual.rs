use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{ProgressiveMesh, ProgressivePoint, ProgressiveWindow};
use xrf_math::{EPS_L, Vector2d, Vector3d};

use crate::geometry::visual::geometry_fast_path::GeometryFastPath;
use crate::geometry::visual::geometry_visual_light::GeometryVisualLight;
use crate::geometry::visual::geometry_visual_vertex::GeometryVisualVertex;

/// One visual as xrLC builds it from a split (`Flex2OGF`): its dress, its welded vertices and faces, and the fast
/// path's positions for an opaque surface.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryVisual {
  /// The `build.prj` material of its faces.
  pub material: u16,
  pub sector: u16,
  /// Its textures as the shader table names them: the base, then `lmap#N_1` and `lmap#N_2`, or the base's `_lm`.
  pub textures: Vec<String>,
  /// Its shader table entry, `<render shader>/<texture>,<texture>...` (`OGF::Save`).
  pub shader: String,
  pub light: GeometryVisualLight,
  pub vertices: Vec<GeometryVisualVertex>,
  pub faces: Vec<[u16; 3]>,
  /// The fast path's positions and faces (`fast_path_data`), empty for a surface with alpha.
  pub fast_vertices: Vec<Vector3d>,
  pub fast_faces: Vec<[u16; 3]>,
  /// A progressive visual's windows over its faces, and over its fast path's where it has one; none for a static one.
  pub windows: Option<Vec<ProgressiveWindow>>,
  pub fast_windows: Option<Vec<ProgressiveWindow>>,
}

impl GeometryVisual {
  /// Fewer faces than this are not worth making progressive (`c_PM_FaceLimit`).
  pub const PROGRESSIVE_FACE_LIMIT: usize = 128;

  /// The share of a static visual's vertices its windows must step through for it to be made progressive
  /// (`c_PM_MetricLimit_static`).
  pub const STATIC_METRIC_LIMIT: f32 = 0.10;

  /// The same for a multi-use model's visual (`c_PM_MetricLimit_mu`).
  pub const MODEL_METRIC_LIMIT: f32 = 0.05;

  /// A visual of no geometry yet.
  pub fn new(material: u16, sector: u16, textures: Vec<String>, shader: String, light: GeometryVisualLight) -> Self {
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
  /// ([`ProgressiveMesh`]) where its windows step through at least `metric_limit` of its vertices. Its fast path
  /// follows over positions alone, with no limit; where that fails, the visual stays as it was.
  pub fn make_progressive(&mut self, metric_limit: f32) {
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

  /// `OGF::_BuildFace`: each corner welded to the first similar vertex ([`Self::is_similar`]) or added. A degenerate
  /// face drops the vertices it added; a repeated one is left out, its vertices kept. `texel` is one texel of each
  /// coordinate channel's texture.
  ///
  /// # Errors
  ///
  /// Returns an error for a 65,537th vertex, past the 16-bit indices, where xrLC wraps onto another vertex.
  pub fn add_face(
    &mut self,
    corners: [GeometryVisualVertex; 3],
    texel: &[Vector2d],
    fast_path: GeometryFastPath,
  ) -> XrfResult {
    let held: usize = self.vertices.len();
    let [first, second, third] = corners;
    let face: [u16; 3] = [
      self.weld(first, texel)?,
      self.weld(second, texel)?,
      self.weld(third, texel)?,
    ];

    if Self::is_degenerate(&face) {
      self.vertices.truncate(held);
      return Ok(());
    }

    if self.faces.iter().any(|other| Self::is_same(other, &face)) {
      return Ok(());
    }

    self.faces.push(face);

    if fast_path == GeometryFastPath::Joined {
      self.add_fast_face(corners.map(|corner| corner.position))?;
    }

    Ok(())
  }

  /// `OGF::x_BuildFace`: the corners welded within `EPS_L`, a degenerate face dropping the vertices it added.
  fn add_fast_face(&mut self, [first, second, third]: [Vector3d; 3]) -> XrfResult {
    let held: usize = self.fast_vertices.len();
    let face: [u16; 3] = [self.weld_fast(first)?, self.weld_fast(second)?, self.weld_fast(third)?];

    if Self::is_degenerate(&face) {
      self.fast_vertices.truncate(held);
    } else {
      self.fast_faces.push(face);
    }

    Ok(())
  }

  /// `OGF::_BuildVertex`.
  fn weld(&mut self, corner: GeometryVisualVertex, texel: &[Vector2d]) -> XrfResult<u16> {
    match self
      .vertices
      .iter()
      .position(|vertex| Self::is_similar(vertex, &corner, texel))
    {
      Some(index) => self.get_index(index),
      None => {
        let index: u16 = self.get_index(self.vertices.len())?;

        self.vertices.push(corner);
        Ok(index)
      }
    }
  }

  /// `OGF::x_BuildVertex`: the fast path's vertex within `EPS_L` of `corner`, or `corner` added.
  fn weld_fast(&mut self, corner: Vector3d) -> XrfResult<u16> {
    match self
      .fast_vertices
      .iter()
      .position(|vertex| vertex.similar(&corner, EPS_L))
    {
      Some(index) => self.get_index(index),
      None => {
        let index: u16 = self.get_index(self.fast_vertices.len())?;

        self.fast_vertices.push(corner);
        Ok(index)
      }
    }
  }

  /// A vertex's place as the visual's 16-bit indices hold it: at most 65,535 vertices, the count a 16-bit field holds.
  fn get_index(&self, index: usize) -> XrfResult<u16> {
    u16::try_from(index)
      .ok()
      .filter(|index| *index < u16::MAX)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "A visual of material {} ({}) welds more than 65,535 vertices, past what its 16-bit indices reach",
          self.material, self.shader
        ))
      })
  }

  /// `OGF_Vertex::similar`: position and normal within `EPS_L`, the basis within a step, each coordinate within a
  /// texel, all strictly.
  fn is_similar(first: &GeometryVisualVertex, second: &GeometryVisualVertex, texel: &[Vector2d]) -> bool {
    let near = |one: &Vector2d, other: &Vector2d, texel: &Vector2d| {
      (one.x - other.x).abs() < texel.x && (one.y - other.y).abs() < texel.y
    };

    first.position.similar(&second.position, EPS_L)
      && first.normal.similar(&second.normal, EPS_L)
      && first.basis.is_similar(&second.basis)
      && near(&first.texture_coordinate, &second.texture_coordinate, &texel[0])
      && match (first.lightmap_coordinate, second.lightmap_coordinate) {
        (Some(one), Some(other)) => near(&one, &other, &texel[1]),
        _ => true,
      }
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

  /// `OGF::Optimize` for a compiler shader that optimizes coordinates: each group of faces joined by shared vertices
  /// has its base coordinates moved by the whole tiles nearest the middle of their span. The span's upper end starts
  /// from the smallest positive float, as xrLC's does.
  pub fn shift_texture_coordinates(&mut self) {
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
