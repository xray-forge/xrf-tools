use xrf_geometry::FaceArea;
use xrf_math::{EPS_L, Vector2d, Vector3d};

use crate::geometry::geometry_face::GeometryFace;
use crate::geometry::geometry_mesh::GeometryMesh;

impl GeometryMesh {
  /// The longest edge xrLC leaves whole, `aht_max_edge`: `c_SS_maxsize / 2.5`, 12.8 m.
  pub const LONGEST_EDGE: f32 = 32.0 / 2.5;

  /// The first pass of `CBuild::xrPhase_AdaptiveHT` (`xrLC/xrPhase_AdaptiveHT.cpp:166`), the only one that changes
  /// geometry and the one xrLC runs at every quality: every face with an edge of [`Self::LONGEST_EDGE`] or more is cut
  /// at the middle of its longest, with every other face sharing that edge, until none is left (`u_Tesselate` with
  /// `callback_edge_longest`). Its later passes only light vertices. Returns the faces cut.
  ///
  /// Faces are taken in order, the halves of a cut one appended after them all and so taken in their turn; a face of an
  /// area under `EPS_L` is left whole. The faces sharing an edge are cut in face order, each into two halves on the new
  /// vertex at the edge's middle, with the base coordinate between the edge's two: the half before the edge in the
  /// face's winding first. A half keeps its face's back-face flag and its two outer edges' softness, and the new edge
  /// between the halves is soft. The cut faces are dropped at the end, the others keeping their order.
  pub fn tessellate(&mut self) -> usize {
    let mut around: Vec<Vec<u32>> = self.get_faces_around_vertices();
    let mut is_cut: Vec<bool> = vec![false; self.faces.len()];
    let mut face: usize = 0;

    while face < self.faces.len() {
      if is_cut[face] {
        face += 1;
        continue;
      }

      if let Some([start, end]) = self.find_edge_to_cut(face) {
        let sharing: Vec<u32> = around[start as usize]
          .iter()
          .copied()
          .filter(|other| !is_cut[*other as usize] && self.faces[*other as usize].vertices.contains(&end))
          .collect();
        let middle: u32 = self.vertices.len() as u32;
        let [from, to] = [self.vertices[start as usize], self.vertices[end as usize]];

        self.vertices.push(Self::get_middle(&from, &to));
        around.push(Vec::new());

        for other in sharing {
          is_cut[other as usize] = true;

          for half in self.cut(other as usize, start, end, middle) {
            let index: u32 = self.faces.len() as u32;

            for vertex in half.vertices {
              around[vertex as usize].push(index);
            }

            self.faces.push(half);
            is_cut.push(false);
          }
        }
      }

      face += 1;
    }

    let cut: usize = is_cut.iter().filter(|is_cut| **is_cut).count();
    let mut index: usize = 0;

    self.faces.retain(|_| {
      index += 1;
      !is_cut[index - 1]
    });

    cut
  }

  /// The edge of face `face` to cut, as its two vertices: its longest of [`Self::LONGEST_EDGE`] or more, the first of
  /// equals (`callback_edge_longest`); none for a face too small to cut.
  fn find_edge_to_cut(&self, face: usize) -> Option<[u32; 2]> {
    let face: &GeometryFace = &self.faces[face];

    if FaceArea::of(&face.get_corners(&self.vertices)) < EPS_L {
      return None;
    }

    let mut longest: Option<([u32; 2], f32)> = None;

    for [first, second] in GeometryFace::EDGES {
      let ends: [u32; 2] = [face.vertices[first], face.vertices[second]];
      let length: f32 = self.vertices[ends[0] as usize].distance(&self.vertices[ends[1] as usize]);

      if length >= Self::LONGEST_EDGE && longest.is_none_or(|(_, longest)| length > longest) {
        longest = Some((ends, length));
      }
    }

    longest.map(|(ends, _)| ends)
  }

  /// The two halves face `face` is cut into at `middle`, on its edge from `start` to `end` (`tessalate_faces`).
  fn cut(&self, face: usize, start: u32, end: u32, middle: u32) -> [GeometryFace; 2] {
    let face: &GeometryFace = &self.faces[face];
    let first: usize = face.get_corner(start).expect("a face sharing an edge holds its start");
    let second: usize = face.get_corner(end).expect("a face sharing an edge holds its end");
    let base: usize = 3 - (first + second);
    let corner = |corner: usize| (face.vertices[corner], face.texture_coordinates[corner]);
    let middle: (u32, Vector2d) = (
      middle,
      Self::get_average(&face.texture_coordinates[first], &face.texture_coordinates[second]),
    );
    // `is_CCW`: whether the edge runs from `start` to `end` in the face's own winding.
    let is_forward: bool = (first + 1) % 3 == second;
    let half = |corners: [(u32, Vector2d); 3], softness: [bool; 3]| {
      let mut half: GeometryFace = GeometryFace {
        vertices: corners.map(|(vertex, _)| vertex),
        texture_coordinates: corners.map(|(_, coordinate)| coordinate),
        material: face.material,
        game_material: face.game_material,
        // A new face's flags start all set (`Tface::Tface`), then each is set from its parent.
        smoothing: u32::MAX,
        source: face.source,
      };

      half.set_back_face(face.is_back_face());

      for (edge, is_soft) in softness.into_iter().enumerate() {
        half.set_soft_edge(edge, is_soft);
      }

      half
    };

    if is_forward {
      let (along, after, before) = (
        face.is_soft_edge(first),
        face.is_soft_edge(second),
        face.is_soft_edge(base),
      );

      [
        half([corner(base), corner(first), middle], [before, along, true]),
        half([corner(base), middle, corner(second)], [true, along, after]),
      ]
    } else {
      let (along, after, before) = (
        face.is_soft_edge(second),
        face.is_soft_edge(base),
        face.is_soft_edge(first),
      );

      [
        half([corner(base), middle, corner(first)], [true, along, before]),
        half([corner(base), corner(second), middle], [after, along, true]),
      ]
    }
  }

  /// `Fvector::lerp` at a half: each end weighed by a half and summed.
  fn get_middle(from: &Vector3d, to: &Vector3d) -> Vector3d {
    Vector3d::new(
      from.x * 0.5 + to.x * 0.5,
      from.y * 0.5 + to.y * 0.5,
      from.z * 0.5 + to.z * 0.5,
    )
  }

  /// `Fvector2::averageA`, in its own single-precision sum.
  #[expect(
    clippy::manual_midpoint,
    reason = "`f32::midpoint` rounds through wider arithmetic, where xrLC sums in single precision"
  )]
  fn get_average(first: &Vector2d, second: &Vector2d) -> Vector2d {
    Vector2d::new((first.x + second.x) * 0.5, (first.y + second.y) * 0.5)
  }
}
