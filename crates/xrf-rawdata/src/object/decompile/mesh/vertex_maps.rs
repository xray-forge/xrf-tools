use std::collections::HashMap;

use xrf_editable_object::{EditableVMap, EditableVMapKind, EditableVMapPoint, EditableVMapReference};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector2d;

use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::mesh_assembler::MeshAssembler;
use crate::object::decompile::mesh::mesh_kind::MeshKind;

/// A mesh's vertex maps as they fill, and the references its corners make into them.
///
/// The maps come in a fixed order: the point UV map first, one weight map per bone some point follows, in bone order,
/// then the polygon UV map, written only where some corner's UV is not its point's. A corner refers to its UV and to
/// its point's entry in the weight map of each bone the point follows, as `CEditableMesh::GenerateSVertices` reads
/// them back.
pub(crate) struct VertexMaps {
  /// The UV of each point, from the first corner standing on it.
  point_uvs: Vec<Option<Vector2d>>,
  polygon_uvs: Vec<f32>,
  polygon_points: Vec<u32>,
  polygon_faces: Vec<u32>,
  /// The bone each weight map is for.
  weight_bones: Vec<u16>,
  weight_points: Vec<Vec<u32>>,
  weight_values: Vec<Vec<f32>>,
  /// Each point's entries in the weight maps of the bones it follows.
  point_weights: Vec<Vec<EditableVMapPoint>>,
  bone_names: Vec<String>,
  uv_map_name: &'static str,
  pub(crate) references: Vec<EditableVMapReference>,
  /// The reference of each UV entry and point a corner has referred to.
  reference_of: HashMap<[u32; 3], u32>,
}

impl VertexMaps {
  const POINT_UV_MAP: u32 = 0;
  const FIRST_WEIGHT_MAP: u32 = 1;

  /// The maps of `welded`'s points, each weighed to the bones it follows, named as the SDK names a mesh of `kind`'s.
  ///
  /// # Errors
  ///
  /// Returns an error when a point's bone is past those `kind` names, or a count outgrows a `u32`.
  pub(crate) fn new(welded: &WeldedMesh, kind: MeshKind<'_>) -> XrfResult<Self> {
    let bone_names: &[String] = kind.get_bone_names();
    let mut weight_bones: Vec<u16> = welded
      .point_influences
      .iter()
      .flat_map(|influences| influences.iter().map(|(bone, _)| bone))
      .collect();

    weight_bones.sort_unstable();
    weight_bones.dedup();

    if let Some(bone) = weight_bones.iter().find(|bone| usize::from(**bone) >= bone_names.len()) {
      return Err(XrfError::new_invalid_error(format!(
        "A vertex is linked to bone {bone}, past the OGF's {} bones",
        bone_names.len()
      )));
    }

    let mut weight_points: Vec<Vec<u32>> = vec![Vec::new(); weight_bones.len()];
    let mut weight_values: Vec<Vec<f32>> = vec![Vec::new(); weight_bones.len()];
    let mut point_weights: Vec<Vec<EditableVMapPoint>> = Vec::with_capacity(welded.point_influences.len());

    for (point, influences) in welded.point_influences.iter().enumerate() {
      let mut entries: Vec<EditableVMapPoint> = Vec::new();

      for (bone, weight) in influences.iter() {
        let map: usize = weight_bones
          .binary_search(&bone)
          .expect("every bone a point follows has a weight map");

        entries.push(EditableVMapPoint {
          vmap: Self::FIRST_WEIGHT_MAP + MeshAssembler::narrow_index(map, "weight maps")?,
          index: MeshAssembler::narrow_index(weight_points[map].len(), "weights")?,
        });
        weight_points[map].push(MeshAssembler::narrow_index(point, "points")?);
        weight_values[map].push(weight);
      }

      point_weights.push(entries);
    }

    Ok(Self {
      point_uvs: vec![None; welded.points.len()],
      polygon_uvs: Vec::new(),
      polygon_points: Vec::new(),
      polygon_faces: Vec::new(),
      weight_bones,
      weight_points,
      weight_values,
      point_weights,
      bone_names: bone_names.to_vec(),
      uv_map_name: kind.get_uv_map_name(),
      references: Vec::new(),
      reference_of: HashMap::new(),
    })
  }

  /// The reference of a corner standing on `point` of `face` with `uv`, shared with every corner that refers alike.
  ///
  /// # Errors
  ///
  /// Returns an error when a count outgrows a `u32`.
  pub(crate) fn refer(&mut self, point: u32, face: u32, uv: Vector2d) -> XrfResult<u32> {
    let point_index: usize = usize::try_from(point).expect("a point index fits the platform");
    let uv_entry: EditableVMapPoint = match self.point_uvs[point_index] {
      None => {
        self.point_uvs[point_index] = Some(uv);

        EditableVMapPoint {
          vmap: Self::POINT_UV_MAP,
          index: point,
        }
      }
      Some(point_uv) if point_uv.to_array().map(f32::to_bits) == uv.to_array().map(f32::to_bits) => EditableVMapPoint {
        vmap: Self::POINT_UV_MAP,
        index: point,
      },
      Some(_) => {
        let index: u32 = MeshAssembler::narrow_index(self.polygon_points.len(), "polygon UVs")?;

        self.polygon_uvs.extend(uv.to_array());
        self.polygon_points.push(point);
        self.polygon_faces.push(face);

        EditableVMapPoint {
          vmap: self.get_polygon_map()?,
          index,
        }
      }
    };
    // A point's weight entries are its own, so the UV entry and the point decide the reference.
    let key: [u32; 3] = [uv_entry.vmap, uv_entry.index, point];

    if let Some(reference) = self.reference_of.get(&key) {
      return Ok(*reference);
    }

    let reference: u32 = MeshAssembler::narrow_index(self.references.len(), "vertex map references")?;
    let mut entries: Vec<EditableVMapPoint> = Vec::with_capacity(1 + self.point_weights[point_index].len());

    entries.push(uv_entry);
    entries.extend_from_slice(&self.point_weights[point_index]);

    self.references.push(EditableVMapReference { points: entries });
    self.reference_of.insert(key, reference);

    Ok(reference)
  }

  /// The maps, in their fixed order.
  pub(crate) fn into_vmaps(self) -> Vec<EditableVMap> {
    let mut vmaps: Vec<EditableVMap> = vec![EditableVMap {
      name: String::from(self.uv_map_name),
      dimension: 2,
      kind: EditableVMapKind::Uv,
      values: self
        .point_uvs
        .iter()
        .flat_map(|uv| uv.unwrap_or(Vector2d::ZERO).to_array())
        .collect(),
      vertex_indices: (0..).take(self.point_uvs.len()).collect(),
      polygon_indices: None,
    }];

    for ((bone, points), values) in self.weight_bones.iter().zip(self.weight_points).zip(self.weight_values) {
      vmaps.push(EditableVMap {
        name: self.bone_names[usize::from(*bone)].clone(),
        dimension: 1,
        kind: EditableVMapKind::Weight,
        values,
        vertex_indices: points,
        polygon_indices: None,
      });
    }

    if !self.polygon_points.is_empty() {
      vmaps.push(EditableVMap {
        name: String::from(self.uv_map_name),
        dimension: 2,
        kind: EditableVMapKind::Uv,
        values: self.polygon_uvs,
        vertex_indices: self.polygon_points,
        polygon_indices: Some(self.polygon_faces),
      });
    }

    vmaps
  }

  /// The polygon UV map's place, after every weight map.
  fn get_polygon_map(&self) -> XrfResult<u32> {
    Ok(Self::FIRST_WEIGHT_MAP + MeshAssembler::narrow_index(self.weight_bones.len(), "weight maps")?)
  }
}
