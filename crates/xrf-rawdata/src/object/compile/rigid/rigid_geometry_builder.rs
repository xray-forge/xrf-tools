use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Mesh, MeshVertex, SplitPartition, TangentBasis, VertexPacker, VertexPacking};
use xrf_math::Aabb;

use crate::object::compile::geometry::object_bounds::ObjectBounds;
use crate::object::compile::geometry::source::mesh_surface_faces::MeshSurfaceFaces;
use crate::object::compile::rigid::rigid_corner::RigidCorner;
use crate::object::compile::rigid::rigid_geometry::RigidGeometry;
use crate::object::compile::rigid::rigid_split::RigidSplit;

/// Builds a static object's splits as `CExportObjectOGF::Prepare` does, its tangent pass included.
///
/// Each surface packs into a split of its own (`FindSplit`, by surface where the skeleton export splits by texture and
/// shader), in [`MeshSurfaceFaces`] order over a grid of [`VertexPacking::RIGID`], a two-sided face again mirrored. The
/// tangent pass ([`TangentBasis`]) still splits and orders the vertices though the OGF keeps no tangents, and a split
/// that outgrows one child is cut ([`SplitPartition`]).
///
/// Two export slips are not followed: its second walk of a two-sided surface writes every face and mirror again, here
/// written once; and its new part at 65,535 vertices loses the face being packed, where here a split is cut after the
/// tangent pass.
pub(crate) struct RigidGeometryBuilder {
  splits: Vec<PendingSplit>,
  bounds: Aabb,
}

/// A surface's split as its faces pack.
struct PendingSplit {
  surface: usize,
  texture: String,
  shader: String,
  packer: VertexPacker,
}

impl RigidGeometryBuilder {
  /// The splits of `object`.
  ///
  /// # Errors
  ///
  /// Returns an error where the export fails: no mesh, a surface the object does not have or with other than one
  /// texture coordinate set, a corner outside the object's box; and where a mesh does not hold together.
  pub(crate) fn build(object: &EditableObjectFile) -> XrfResult<RigidGeometry> {
    let meshes: &[EditableMesh] = object.meshes.as_deref().unwrap_or_default();
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();
    let Some(bounds) = ObjectBounds::of(meshes) else {
      return Err(XrfError::new_invalid_error("A static object holds no mesh to export"));
    };
    let mut builder: Self = Self {
      splits: Vec::new(),
      bounds,
    };

    for mesh in meshes {
      let corners: Vec<[RigidCorner; 3]> = RigidCorner::of_mesh(mesh)?;

      for walked in MeshSurfaceFaces::in_export_order(mesh, surfaces)? {
        let split: usize = builder.find_split(walked.index, walked.surface);

        for face in walked.faces {
          builder.splits[split].add_face(walked.surface, &corners[*face as usize])?;
        }
      }
    }

    builder.finish()
  }

  /// The split of surface `index`, made when first needed.
  fn find_split(&mut self, index: usize, surface: &EditableSurface) -> usize {
    self
      .splits
      .iter()
      .position(|split| split.surface == index)
      .unwrap_or_else(|| {
        self.splits.push(PendingSplit {
          surface: index,
          texture: surface.texture.clone(),
          shader: surface.shader.clone(),
          packer: VertexPacker::new(&self.bounds, &VertexPacking::RIGID),
        });

        self.splits.len() - 1
      })
  }

  /// The splits that kept a face, each through the tangent pass and cut where it outgrows one child.
  fn finish(self) -> XrfResult<RigidGeometry> {
    let mut dropped_faces: usize = 0;
    let mut splits: Vec<RigidSplit> = Vec::with_capacity(self.splits.len());

    for split in self.splits {
      dropped_faces += split.packer.get_dropped_faces();

      if split.packer.get_faces().is_empty() {
        continue;
      }

      let mended: Mesh<u32> = TangentBasis::mend(split.packer.into_mesh())?;

      for Mesh { vertices, faces } in SplitPartition::cut(mended) {
        splits.push(RigidSplit {
          texture: split.texture.clone(),
          shader: split.shader.clone(),
          vertices,
          faces,
        });
      }
    }

    Ok(RigidGeometry { splits, dropped_faces })
  }
}

impl PendingSplit {
  /// Packs a face of `corners`, and where `surface` is two-sided its mirror: wound the other way, its normals turned
  /// around, from the corners as packing left them.
  fn add_face(&mut self, surface: &EditableSurface, corners: &[RigidCorner; 3]) -> XrfResult {
    let mut vertices: [MeshVertex; 3] = corners.each_ref().map(RigidCorner::to_vertex);

    self.packer.add_face(&mut vertices)?;

    if surface.is_two_sided() {
      for vertex in &mut vertices {
        vertex.normal = -&vertex.normal;
      }

      let [first, second, third] = vertices;

      self.packer.add_face(&mut [third, second, first])?;
    }

    Ok(())
  }
}
