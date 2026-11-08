use std::collections::{HashMap, HashSet};

use xrf_math::Vector3d;
use xrf_ogf::{OgfFile, OgfVertexData};

use crate::object::verify::material_vertex::MaterialVertex;

/// A vertex's position bits: the export copies one source point into every vertex standing at it, so equal bits pair a
/// compiled vertex with a reference one and never merge two that are merely close.
pub(crate) type PositionKey = [u32; 3];

/// Everything an OGF draws with one texture and shader, over every child drawing with them, as the comparison reads it:
/// its vertices and the faces it draws at full detail.
///
/// Where one surface's faces fall among children is the exporter's choice: the export splits a surface by breakable
/// part, the compile cuts one past 16-bit indices, and other tools spread one over several children. The engine draws
/// any of these alike, so the comparison takes a texture and shader's children together.
pub(crate) struct OgfMaterialGeometry<'a> {
  pub(crate) texture: &'a str,
  pub(crate) shader: &'a str,
  /// The children drawing with them.
  pub(crate) children: usize,
  /// Their vertices, child after child.
  pub(crate) vertices: Vec<MaterialVertex>,
  /// Their faces over [`Self::vertices`].
  pub(crate) faces: Vec<[u32; 3]>,
}

/// How a compiled texture and shader's geometry compares with the reference's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OgfMaterialComparison {
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

impl<'a> OgfMaterialGeometry<'a> {
  /// Normals within this of each other are one normal.
  const NORMAL_TOLERANCE: f32 = 1e-3;
  /// Texture coordinates this close to a whole number of tiles apart are one coordinate.
  const TEXTURE_TOLERANCE: f32 = 1e-4;
  /// Weights within this of each other are one weight.
  const WEIGHT_TOLERANCE: f32 = 1e-3;

  /// The geometry of each texture and shader that `file` and its children draw, skinned or static, in the order each
  /// is first stored.
  pub(crate) fn collect(file: &'a OgfFile) -> Vec<Self> {
    let mut materials: Vec<Self> = Vec::new();

    Self::collect_into(file, &mut materials);

    materials
  }

  fn collect_into(file: &'a OgfFile, materials: &mut Vec<Self>) {
    if let (Some(texture), Some(geometry)) = (&file.texture, &file.geometry) {
      let vertices: Vec<MaterialVertex> = match &geometry.vertices {
        Some(OgfVertexData::Skinned(vertices)) => vertices.iter().map(MaterialVertex::from).collect(),
        Some(OgfVertexData::Static(vertices)) => vertices.iter().map(MaterialVertex::from).collect(),
        None => Vec::new(),
      };
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
      let index: usize = materials
        .iter()
        .position(|it| it.texture == texture.texture_name && it.shader == texture.shader_name)
        .unwrap_or_else(|| {
          materials.push(Self {
            texture: &texture.texture_name,
            shader: &texture.shader_name,
            children: 0,
            vertices: Vec::new(),
            faces: Vec::new(),
          });

          materials.len() - 1
        });
      let material: &mut Self = &mut materials[index];
      let first: u32 = u32::try_from(material.vertices.len()).expect("an OGF's vertices are numbered in 32 bits");

      material.children += 1;
      // A face naming a vertex its child lacks draws nothing, and would name another child's once numbered after it.
      material.faces.extend(
        drawn
          .as_chunks::<3>()
          .0
          .iter()
          .filter(|face| face.iter().all(|vertex| usize::from(*vertex) < vertices.len()))
          .map(|face| face.map(|vertex| first + u32::from(vertex))),
      );
      material.vertices.extend(vertices);
    }

    for child in file.children.iter().flat_map(|children| &children.nested) {
      Self::collect_into(child, materials);
    }
  }

  /// How this compiled geometry compares with `reference`: its vertices by position, normal, texture coordinate and
  /// weights, and the faces either draws by their corners' positions in winding order.
  ///
  /// Each face is compared once: a face drawn twice draws nothing more, as the static export's second walk of a
  /// two-sided surface writes every face again, while its mirror, wound the other way, is a face of its own. A face
  /// with two corners at one position draws nothing, so it is compared on neither side: the export leaves one where
  /// `VPack` moved a corner onto another, which either side may have done where the other did not.
  pub(crate) fn compare(&self, reference: &Self) -> OgfMaterialComparison {
    let mut comparison: OgfMaterialComparison = OgfMaterialComparison::default();
    let by_position: HashMap<PositionKey, Vec<&MaterialVertex>> = reference.by_position();

    for vertex in &self.vertices {
      let candidates: &[&MaterialVertex] = by_position
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

    let mut drawn: HashSet<[PositionKey; 3]> = self.triangles().into_iter().filter(|it| !Self::is_flat(it)).collect();

    for triangle in reference.triangles() {
      if Self::is_flat(&triangle) {
        comparison.reference_flat_faces += 1;
      } else if drawn.remove(&triangle) {
        comparison.faces_matched += 1;
      } else {
        comparison.faces_missing += 1;
      }
    }

    comparison.faces_extra = drawn.len();

    comparison
  }

  /// Whether a triangle has two corners at one position, so draws nothing.
  fn is_flat(triangle: &[PositionKey; 3]) -> bool {
    triangle[0] == triangle[1] || triangle[1] == triangle[2] || triangle[0] == triangle[2]
  }

  pub(crate) fn position_key(position: &Vector3d) -> PositionKey {
    [position.x.to_bits(), position.y.to_bits(), position.z.to_bits()]
  }

  /// Its vertices by where they stand.
  fn by_position(&self) -> HashMap<PositionKey, Vec<&MaterialVertex>> {
    let mut by_position: HashMap<PositionKey, Vec<&MaterialVertex>> = HashMap::new();

    for vertex in &self.vertices {
      by_position
        .entry(Self::position_key(&vertex.position))
        .or_default()
        .push(vertex);
    }

    by_position
  }

  /// Its distinct faces as the positions of their corners in winding order, each turned to start at its lowest
  /// position, so a face matches whatever corner it starts at and its vertices' numbering, and its mirror does not; a
  /// face naming a vertex past them is left out. In the order first drawn.
  pub(crate) fn triangles(&self) -> Vec<[PositionKey; 3]> {
    let mut seen: HashSet<[PositionKey; 3]> = HashSet::new();

    self
      .faces
      .iter()
      .filter_map(|face| {
        let [a, b, c] = face.map(|index| {
          self
            .vertices
            .get(index as usize)
            .map(|vertex| Self::position_key(&vertex.position))
        });
        let corners: [PositionKey; 3] = [a?, b?, c?];
        let start: usize = (0..3).min_by_key(|index| corners[*index]).unwrap_or(0);
        let rotated: [PositionKey; 3] = std::array::from_fn(|index| corners[(start + index) % 3]);

        seen.insert(rotated).then_some(rotated)
      })
      .collect()
  }

  /// Whether two vertices standing at one position are one vertex to the engine: the same normal, and alike
  /// [apart from it](Self::is_same_vertex_apart_from_normal).
  pub(crate) fn is_same_vertex(first: &MaterialVertex, second: &MaterialVertex) -> bool {
    first
      .normal
      .to_array()
      .iter()
      .zip(second.normal.to_array())
      .all(|(a, b)| (a - b).abs() <= Self::NORMAL_TOLERANCE)
      && Self::is_same_vertex_apart_from_normal(first, second)
  }

  /// Whether two vertices standing at one position have the same texture coordinate modulo whole tiles, and the same
  /// bones with their weights: drawn and moved alike, perhaps shaded otherwise.
  pub(crate) fn is_same_vertex_apart_from_normal(first: &MaterialVertex, second: &MaterialVertex) -> bool {
    let is_whole_tiles = |first: f32, second: f32| {
      let delta: f32 = first - second;

      (delta - delta.round()).abs() <= Self::TEXTURE_TOLERANCE
    };

    is_whole_tiles(first.texture_coordinate.x, second.texture_coordinate.x)
      && is_whole_tiles(first.texture_coordinate.y, second.texture_coordinate.y)
      && first.bones.len() == second.bones.len()
      && first
        .bones
        .iter()
        .zip(&second.bones)
        .all(|(own, other)| own.bone == other.bone && (own.weight - other.weight).abs() <= Self::WEIGHT_TOLERANCE)
  }
}
