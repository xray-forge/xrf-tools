use xrf_math::Vector3d;

use crate::data::visual::geometry::visual_clusters::VisualClusters;
use crate::data::visual::geometry::visual_geometry::VisualGeometry;
use crate::data::visual::geometry::visual_submesh::VisualSubmesh;
use crate::data::visual::geometry::visual_submesh_content::VisualSubmeshContent;
use crate::data::visual::skeleton::visual_rest_pose::VisualRestPose;
use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::data::visual::visual_description::VisualDescription;
use crate::pack::visual::visual_package::VisualPackage;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_cluster_table::VisualClusterTable;

/// Links a skinned vertex hangs from.
const LINKS: usize = 4;

/// Stands a packed visual still in a pose, as static geometry: every skinned vertex moved by the bones it hangs from,
/// from their bind to the pose, weighted as its skin weights it, and every submesh cut into clusters as geometry drawn
/// as stored is.
pub struct VisualPoser;

impl VisualPoser {
  /// The visual standing in `pose`. A submesh without skin, or a visual without a pose of its own bones or their binds,
  /// stands as stored.
  pub fn pose(package: &VisualPackage, pose: Option<&VisualRestPose>) -> VisualPackage {
    let skins: Option<Vec<VisualTransform>> = pose.and_then(|pose| Self::get_skins(&package.description, pose));
    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
    let submeshes: Vec<VisualSubmesh> = package
      .description
      .submeshes
      .iter()
      .map(|submesh| VisualSubmesh {
        content: match &submesh.content {
          VisualSubmeshContent::Packed { geometry } => VisualSubmeshContent::Packed {
            geometry: Self::pose_geometry(geometry, package, skins.as_deref(), &mut builder),
          },
          skipped => skipped.clone(),
        },
        ..submesh.clone()
      })
      .collect();
    let buffer: Vec<u8> = builder.into_buffer();

    VisualPackage {
      description: VisualDescription {
        submeshes,
        buffer_length: buffer.len() as u32,
        ..package.description.clone()
      },
      buffer,
    }
  }

  /// Each bone's pose after the inverse of its bind: what takes a bound vertex to where the bone stands. `None` where a
  /// bone carries no bind, or the pose stands another count of bones: the engine poses every bone, and the packer has
  /// held each link to one.
  fn get_skins(description: &VisualDescription, pose: &VisualRestPose) -> Option<Vec<VisualTransform>> {
    if pose.transforms.len() != description.bones.len() {
      return None;
    }

    description
      .bones
      .iter()
      .zip(&pose.transforms)
      .map(|(bone, posed)| bone.bind_transform.as_ref().map(|bind| bind.inverse().then(posed)))
      .collect()
  }

  fn pose_geometry(
    geometry: &VisualGeometry,
    package: &VisualPackage,
    skins: Option<&[VisualTransform]>,
    builder: &mut VisualBufferBuilder,
  ) -> VisualGeometry {
    let mut positions: Vec<f32> = package.read_f32_section(geometry.positions);
    let mut normals: Vec<f32> = package.read_f32_section(geometry.normals);
    let mut tangents: Vec<f32> = package.read_f32_section(geometry.tangents);
    let mut binormals: Vec<f32> = package.read_f32_section(geometry.binormals);
    let uvs: Vec<f32> = package.read_f32_section(geometry.uvs);
    let indices: Vec<u16> = package.read_u16_section(geometry.indices);

    if let (Some(skin), Some(skins)) = (&geometry.skin, skins) {
      let links: Vec<u16> = package.read_u16_section(skin.indices);
      let weights: Vec<f32> = package.read_f32_section(skin.weights);

      for vertex in 0..(positions.len() / 3)
        .min(links.len() / LINKS)
        .min(weights.len() / LINKS)
      {
        let linked: Vec<(&VisualTransform, f32)> = (0..LINKS)
          .filter_map(|link| {
            let weight: f32 = weights[vertex * LINKS + link];

            skins
              .get(links[vertex * LINKS + link] as usize)
              .filter(|_| weight != 0.0)
              .map(|skin| (skin, weight))
          })
          .collect();

        Self::move_point(&mut positions, vertex, &linked);

        for directions in [&mut normals, &mut tangents, &mut binormals] {
          Self::turn_direction(directions, vertex, &linked);
        }
      }
    }

    let points: Vec<[f32; 3]> = positions
      .as_chunks::<3>()
      .0
      .iter()
      .map(|it| [it[0], it[1], it[2]])
      .collect();
    let cut: Vec<u32> = indices.iter().map(|index| u32::from(*index)).collect();
    let mut table: VisualClusterTable = VisualClusterTable::default();

    for level in &geometry.detail_levels {
      table.push_run(&cut, &points, level.start, level.count, VisualClusters::NO_DRAWABLE);
    }

    VisualGeometry {
      positions: builder.push_f32_section(&positions),
      normals: builder.push_f32_section(&normals),
      tangents: builder.push_f32_section(&tangents),
      binormals: builder.push_f32_section(&binormals),
      uvs: builder.push_f32_section(&uvs),
      indices: builder.push_u16_section(&indices),
      skin: None,
      clusters: Some(table.write_into(builder)),
      ..geometry.clone()
    }
  }

  /// A point moved by each bone it hangs from, weighted, in place.
  fn move_point(values: &mut [f32], vertex: usize, linked: &[(&VisualTransform, f32)]) {
    let point: Vector3d = Vector3d::new(values[vertex * 3], values[vertex * 3 + 1], values[vertex * 3 + 2]);
    let mut moved: Vector3d = Vector3d::ZERO;

    for (skin, weight) in linked.iter().copied() {
      moved += &(&(&skin.rotate(&point) + &skin.c) * weight);
    }

    values[vertex * 3..vertex * 3 + 3].copy_from_slice(&moved.to_array());
  }

  /// A direction turned by each bone it hangs from, weighted, and made a unit again, in place; one that sums to
  /// nothing stays as it was.
  fn turn_direction(values: &mut [f32], vertex: usize, linked: &[(&VisualTransform, f32)]) {
    if values.len() < vertex * 3 + 3 {
      return;
    }

    let direction: Vector3d = Vector3d::new(values[vertex * 3], values[vertex * 3 + 1], values[vertex * 3 + 2]);
    let mut turned: Vector3d = Vector3d::ZERO;

    for (skin, weight) in linked.iter().copied() {
      turned += &(&skin.rotate(&direction) * weight);
    }

    let length: f32 = turned.length();

    if length > 0.0 {
      values[vertex * 3..vertex * 3 + 3].copy_from_slice(&(&turned / length).to_array());
    }
  }
}
