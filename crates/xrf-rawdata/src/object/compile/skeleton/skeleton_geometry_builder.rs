use std::collections::HashMap;

use xrf_editable_object::{EditableBone, EditableMesh, EditableObjectFile, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};

use crate::object::compile::geometry::export_vertex::ExportVertex;
use crate::object::compile::geometry::export_vertex_bone::ExportVertexBone;
use crate::object::compile::geometry::source::mesh_surface_faces::MeshSurfaceFaces;
use crate::object::compile::geometry::split_partition::SplitPartition;
use crate::object::compile::geometry::tangent_basis::TangentBasis;
use crate::object::compile::geometry::vertex_packer::VertexPacker;
use crate::object::compile::geometry::vertex_packing::VertexPacking;
use crate::object::compile::object_compile_options::{ObjectCompileOptions, TangentMethod};
use crate::object::compile::skeleton::breakable_parts::BreakableParts;
use crate::object::compile::skeleton::mikktspace_basis::MikkTSpaceBasis;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;
use crate::object::compile::skeleton::skin_corner::SkinCorner;
use crate::object::compile::skeleton::vertex_cache_order::VertexCacheOrder;

/// Builds a skeleton object's splits as `CExportSkeleton::PrepareGeometry` does, its tangent pass included.
///
/// Every mesh's faces are taken surface by surface in the object's surface order, the closest record of the order the
/// export walks them in (their addresses in memory), each surface's faces as the mesh lists them. A face's link count
/// is its widest corner's; a corner of a face whose corners all follow one bone lists that bone twice, the copy
/// weightless, and any other corner lists its own bones, each sorted by bone. The face goes to the split of its
/// surface's engine shader and texture and its bones' breakable part, made in the order first needed, and a two-sided
/// surface's face goes again mirrored, its normals turned around. Each split then goes through the tangent pass
/// ([`TangentBasis`], or [`MikkTSpaceBasis`] where the options ask), and its triangles and vertices are ordered for the
/// vertex cache ([`VertexCacheOrder`]).
pub(crate) struct SkeletonGeometryBuilder {
  splits: Vec<(SkeletonSplit, VertexPacker)>,
  bounds: Aabb,
  parts: Vec<u16>,
  is_breakable: bool,
}

impl SkeletonGeometryBuilder {
  /// Bones a skeleton object can hold (`PrepareGeometry`).
  const MAX_BONES: usize = 64;

  /// The splits of `object`.
  ///
  /// # Errors
  ///
  /// Returns an error where the export fails: no mesh, no bone or more than [`Self::MAX_BONES`], a surface the object
  /// does not have or with other than one texture coordinate set, a corner following no bone, a face spanning two
  /// breakable parts; and where a mesh does not hold together.
  pub(crate) fn build(object: &EditableObjectFile, options: &ObjectCompileOptions) -> XrfResult<SkeletonGeometry> {
    let meshes: &[EditableMesh] = object.meshes.as_deref().unwrap_or_default();
    let bones: &[EditableBone] = object.bones.as_deref().unwrap_or_default();
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();

    let Some(bounds) = VertexPacker::get_object_bounds(meshes) else {
      return Err(XrfError::new_invalid_error("A skeleton object holds no mesh to export"));
    };

    if bones.is_empty() || bones.len() > Self::MAX_BONES {
      return Err(XrfError::new_invalid_error(format!(
        "A skeleton object exports with 1 to {} bones, not {}",
        Self::MAX_BONES,
        bones.len()
      )));
    }

    // `GetBoneIndexByWMap` takes the first bone a weight map names.
    let mut bone_of_map: HashMap<&str, u16> = HashMap::with_capacity(bones.len());

    for (bone, index) in bones.iter().zip(0..) {
      bone_of_map.entry(bone.definition.weight_map.as_str()).or_insert(index);
    }
    let parts: Vec<u16> = BreakableParts::of(bones);
    let mut builder: Self = Self {
      splits: Vec::new(),
      bounds,
      is_breakable: parts.iter().any(|part| *part != 0),
      parts,
    };

    for mesh in meshes {
      let corners: Vec<[SkinCorner; 3]> = SkinCorner::of_mesh(mesh, &bone_of_map)?;

      for walked in MeshSurfaceFaces::in_export_order(mesh, surfaces)? {
        for face in walked.faces {
          builder.add_face(walked.surface, &corners[*face as usize])?;
        }
      }
    }

    builder.finish(options.tangents)
  }

  fn add_face(&mut self, surface: &EditableSurface, corners: &[SkinCorner; 3]) -> XrfResult {
    let link_count: usize = corners.iter().map(|corner| corner.weights.len()).max().unwrap_or(0);

    if corners.iter().any(|corner| corner.weights.is_empty()) {
      return Err(XrfError::new_invalid_error(format!(
        "A face of surface '{}' has a corner following no bone",
        surface.name
      )));
    }

    let mut vertices: [ExportVertex; 3] = corners.clone().map(|corner| Self::to_vertex(corner, link_count == 1));
    let part: u16 = self.get_part(&vertices)?;
    let split: usize = self.find_split(surface, part);
    let (split, packer) = &mut self.splits[split];

    split.link_count = split.link_count.max(u8::try_from(link_count).unwrap_or(u8::MAX));
    split.used_bones.extend(
      vertices
        .iter()
        .flat_map(|vertex| vertex.bones.iter().map(|bone| bone.bone)),
    );
    packer.add_face(&mut vertices)?;

    if surface.is_two_sided() {
      for vertex in &mut vertices {
        vertex.normal = -&vertex.normal;
      }

      let [first, second, third] = vertices;

      packer.add_face(&mut [first, third, second])?;
    }

    Ok(())
  }

  /// A corner as the export sets its vertex: its bones sorted by bone, one bone padded with its weightless copy in a
  /// face whose every corner follows one.
  fn to_vertex(corner: SkinCorner, is_one_link: bool) -> ExportVertex {
    let mut bones: Vec<ExportVertexBone> = if is_one_link {
      vec![
        ExportVertexBone {
          bone: corner.weights[0].bone,
          weight: 1.0,
        },
        ExportVertexBone {
          bone: corner.weights[0].bone,
          weight: 0.0,
        },
      ]
    } else {
      corner.weights
    };

    bones.sort_by_key(|bone| bone.bone);

    ExportVertex {
      position: corner.position,
      normal: corner.normal,
      // What `SSkelVert` starts with, until the tangent pass.
      tangent: Vector3d::X,
      binormal: Vector3d::Z,
      texture_coordinate: corner.texture_coordinate,
      bones,
    }
  }

  /// The breakable part a face's bones belong to, all of them one part or the export refuses the object.
  fn get_part(&self, vertices: &[ExportVertex; 3]) -> XrfResult<u16> {
    if !self.is_breakable {
      return Ok(0);
    }

    let mut bones: Vec<u16> = vertices
      .iter()
      .flat_map(|vertex| vertex.bones.iter().map(|bone| bone.bone))
      .collect();

    bones.sort_unstable();
    bones.dedup();

    let part: u16 = self.parts[usize::from(bones[0])];

    if bones.iter().any(|bone| self.parts[usize::from(*bone)] != part) {
      return Err(XrfError::new_invalid_error(
        "A face spans two breakable parts, which the export cannot split",
      ));
    }

    Ok(part)
  }

  /// The split of `surface`'s shader and texture and `part`, made when first needed (`FindSplit`).
  fn find_split(&mut self, surface: &EditableSurface, part: u16) -> usize {
    let found: Option<usize> = self
      .splits
      .iter()
      .position(|(split, _)| split.shader == surface.shader && split.texture == surface.texture && split.part == part);

    found.unwrap_or_else(|| {
      self.splits.push((
        SkeletonSplit {
          shader: surface.shader.clone(),
          texture: surface.texture.clone(),
          part,
          link_count: 0,
          used_bones: Vec::new(),
          vertices: Vec::new(),
          faces: Vec::new(),
        },
        VertexPacker::new(&self.bounds, &VertexPacking::SKELETON),
      ));

      self.splits.len() - 1
    })
  }

  /// The splits that kept a face, each through the tangent pass `tangents` names (see [`TangentBasis`] and
  /// [`MikkTSpaceBasis`]), cut where it outgrows one child (see [`SplitPartition`]), each child ordered for the vertex
  /// cache (see [`VertexCacheOrder`]) and with its bones listed once.
  fn finish(self, tangents: TangentMethod) -> XrfResult<SkeletonGeometry> {
    let mut dropped_faces: usize = 0;
    let mut splits: Vec<SkeletonSplit> = Vec::with_capacity(self.splits.len());

    for (mut split, packer) in self.splits {
      dropped_faces += packer.dropped_faces;

      if packer.faces.is_empty() {
        continue;
      }

      let (vertices, faces) = match tangents {
        TangentMethod::ActorEditor => TangentBasis::mend(packer.vertices, &packer.faces)?,
        TangentMethod::MikkTSpace => MikkTSpaceBasis::generate(packer.vertices, &packer.faces)?,
      };
      let children: Vec<(Vec<ExportVertex>, Vec<[u16; 3]>)> = SplitPartition::cut(vertices, &faces);
      let is_cut: bool = children.len() > 1;

      split.used_bones.sort_unstable();
      split.used_bones.dedup();

      for (vertices, faces) in children {
        let (vertices, faces) = VertexCacheOrder::apply(vertices, &faces);
        // A whole split keeps the bones its faces named, the dropped ones' included, as the export lists them; a cut
        // one's children each list the bones their own vertices follow.
        let used_bones: Vec<u16> = if is_cut {
          let mut bones: Vec<u16> = vertices
            .iter()
            .flat_map(|vertex| vertex.bones.iter().map(|bone| bone.bone))
            .collect();

          bones.sort_unstable();
          bones.dedup();
          bones
        } else {
          split.used_bones.clone()
        };

        splits.push(SkeletonSplit {
          used_bones,
          vertices,
          faces,
          ..split.clone()
        });
      }
    }

    Ok(SkeletonGeometry { splits, dropped_faces })
  }
}
