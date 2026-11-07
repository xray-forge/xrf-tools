use std::collections::{BTreeMap, HashMap};

use xrf_math::Vector3d;
use xrf_ogf::{OgfFile, OgfVertex};

/// A vertex's position bits: the export copies one source point into every vertex standing at it, so equal bits pair a
/// compiled vertex with a reference one and never merge two that are merely close.
pub(crate) type PositionKey = [u32; 3];

/// One drawn child of an OGF as the comparison reads it: what it is drawn with, its vertices, and the faces it draws at
/// full detail.
pub(crate) struct OgfChildGeometry<'a> {
  pub(crate) texture: &'a str,
  pub(crate) shader: &'a str,
  pub(crate) vertices: &'a [OgfVertex],
  pub(crate) faces: Vec<[u16; 3]>,
}

/// How a compiled child compares with the reference child it pairs with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OgfChildComparison {
  /// Compiled vertices one of the reference's stands in for.
  pub(crate) vertices_matched: usize,
  /// Compiled vertices that would match but for their normal.
  pub(crate) vertices_normal_only: usize,
  /// Reference faces the compile draws too.
  pub(crate) faces_matched: usize,
  /// Reference faces with two corners at one position.
  pub(crate) reference_flat_faces: usize,
  /// Reference faces the compile does not draw, and faces it draws the reference does not.
  pub(crate) faces_missing: usize,
  pub(crate) faces_extra: usize,
}

impl<'a> OgfChildGeometry<'a> {
  /// Normals within this of each other are one normal.
  const NORMAL_TOLERANCE: f32 = 1e-3;
  /// Texture coordinates this close to a whole number of tiles apart are one coordinate.
  const TEXTURE_TOLERANCE: f32 = 1e-4;
  /// Weights within this of each other are one weight.
  const WEIGHT_TOLERANCE: f32 = 1e-3;

  /// Every skinned child of `file` and of its children, in the order stored.
  pub(crate) fn collect(file: &'a OgfFile) -> Vec<Self> {
    let mut children: Vec<Self> = Vec::new();

    Self::collect_into(file, &mut children);

    children
  }

  fn collect_into(file: &'a OgfFile, children: &mut Vec<Self>) {
    if let (Some(texture), Some(geometry)) = (&file.texture, &file.geometry)
      && let Some(vertices) = geometry.get_skinned_vertices()
    {
      let indices: &[u16] = geometry.indices.as_deref().unwrap_or_default();
      // A progressive child draws its full detail through its first sliding window.
      let drawn: &[u16] = file
        .swi_data
        .as_ref()
        .and_then(|swi| swi.windows.first())
        .and_then(|window| {
          let start: usize = usize::try_from(window.offset).ok()?;

          indices.get(start..start + usize::from(window.num_tris) * 3)
        })
        .unwrap_or(indices);

      children.push(Self {
        texture: &texture.texture_name,
        shader: &texture.shader_name,
        vertices,
        faces: drawn.as_chunks::<3>().0.to_vec(),
      });
    }

    for child in file.children.iter().flat_map(|children| &children.nested) {
      Self::collect_into(child, children);
    }
  }

  /// How this compiled child compares with `reference`: its vertices by position, normal, texture coordinate and
  /// weights, and the faces either draws by their corners' positions, each as often as it is drawn.
  ///
  /// A face with two corners at one position draws nothing, so it is compared on neither side: the export leaves one
  /// where `VPack` moved a corner onto another, which either side may have done where the other did not.
  pub(crate) fn compare(&self, reference: &Self) -> OgfChildComparison {
    let mut comparison: OgfChildComparison = OgfChildComparison::default();
    let by_position: HashMap<PositionKey, Vec<&OgfVertex>> = reference.by_position();

    for vertex in self.vertices {
      let candidates: &[&OgfVertex] = by_position
        .get(&Self::position_key(&vertex.position))
        .map_or(&[], Vec::as_slice);

      if candidates
        .iter()
        .any(|candidate| Self::is_same_vertex(vertex, candidate))
      {
        comparison.vertices_matched += 1;
      } else if candidates
        .iter()
        .any(|candidate| Self::is_same_vertex_apart_from_normal(vertex, candidate))
      {
        comparison.vertices_normal_only += 1;
      }
    }

    let mut drawn: HashMap<[PositionKey; 3], usize> = HashMap::new();

    for triangle in self.triangles().into_iter().filter(|it| !Self::is_flat(it)) {
      *drawn.entry(triangle).or_default() += 1;
    }

    for triangle in reference.triangles() {
      if Self::is_flat(&triangle) {
        comparison.reference_flat_faces += 1;
      } else if let Some(count) = drawn.get_mut(&triangle)
        && *count > 0
      {
        *count -= 1;
        comparison.faces_matched += 1;
      } else {
        comparison.faces_missing += 1;
      }
    }

    comparison.faces_extra = drawn.values().sum();

    comparison
  }

  /// Whether a triangle of sorted corners has two at one position, so draws nothing.
  fn is_flat(triangle: &[PositionKey; 3]) -> bool {
    triangle[0] == triangle[1] || triangle[1] == triangle[2]
  }

  pub(crate) fn position_key(position: &Vector3d) -> PositionKey {
    [position.x.to_bits(), position.y.to_bits(), position.z.to_bits()]
  }

  /// Its vertices by where they stand.
  fn by_position(&self) -> HashMap<PositionKey, Vec<&'a OgfVertex>> {
    let mut by_position: HashMap<PositionKey, Vec<&'a OgfVertex>> = HashMap::new();

    for vertex in self.vertices {
      by_position
        .entry(Self::position_key(&vertex.position))
        .or_default()
        .push(vertex);
    }

    by_position
  }

  /// Its faces as the positions of their corners, each face's corners sorted, so a face matches whatever its winding
  /// start and its vertices' numbering; a face naming a vertex the child lacks is left out.
  pub(crate) fn triangles(&self) -> Vec<[PositionKey; 3]> {
    self
      .faces
      .iter()
      .filter_map(|face| {
        let [a, b, c] = face.map(|index| {
          self
            .vertices
            .get(usize::from(index))
            .map(|vertex| Self::position_key(&vertex.position))
        });
        let mut corners: [PositionKey; 3] = [a?, b?, c?];

        corners.sort_unstable();

        Some(corners)
      })
      .collect()
  }

  /// Whether two vertices standing at one position are one vertex to the engine: the same normal, and alike
  /// [apart from it](Self::is_same_vertex_apart_from_normal).
  pub(crate) fn is_same_vertex(first: &OgfVertex, second: &OgfVertex) -> bool {
    first
      .normal
      .to_array()
      .iter()
      .zip(second.normal.to_array())
      .all(|(a, b)| (a - b).abs() <= Self::NORMAL_TOLERANCE)
      && Self::is_same_vertex_apart_from_normal(first, second)
  }

  /// Whether two vertices standing at one position have the same texture coordinate modulo whole tiles, and bones with
  /// their weights once each vertex's padding links are summed away: drawn and moved alike, perhaps shaded otherwise.
  pub(crate) fn is_same_vertex_apart_from_normal(first: &OgfVertex, second: &OgfVertex) -> bool {
    let is_whole_tiles = |first: f32, second: f32| {
      let delta: f32 = first - second;

      (delta - delta.round()).abs() <= Self::TEXTURE_TOLERANCE
    };
    let (bones, other_bones) = (Self::collapse(first), Self::collapse(second));

    is_whole_tiles(first.texture_u, second.texture_u)
      && is_whole_tiles(first.texture_v, second.texture_v)
      && bones.len() == other_bones.len()
      && bones
        .iter()
        .zip(&other_bones)
        .all(|((bone, weight), (other_bone, other_weight))| {
          bone == other_bone && (weight - other_weight).abs() <= Self::WEIGHT_TOLERANCE
        })
  }

  /// A vertex's bones summed per bone, those of no weight dropped: a layout pads a narrower vertex with repeated or
  /// empty links, which the engine blends to the same pose.
  fn collapse(vertex: &OgfVertex) -> Vec<(u16, f32)> {
    let mut summed: BTreeMap<u16, f32> = BTreeMap::new();

    for link in vertex.links.iter() {
      *summed.entry(link.bone).or_default() += link.weight;
    }

    summed.into_iter().filter(|(_, weight)| *weight > 1e-6).collect()
  }
}
