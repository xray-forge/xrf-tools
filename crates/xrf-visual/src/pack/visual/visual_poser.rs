use xrf_math::Vector3d;

use crate::data::visual::geometry::visual_clusters::VisualClusters;
use crate::data::visual::geometry::visual_geometry::VisualGeometry;
use crate::data::visual::geometry::visual_section::VisualSection;
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
  /// The visual standing in `pose`. A submesh without skin, or a visual without a pose or its bones' binds, stands as
  /// stored.
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
            geometry: Self::pose_geometry(geometry, &package.buffer, skins.as_deref(), &mut builder),
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
  /// bone carries no bind.
  fn get_skins(description: &VisualDescription, pose: &VisualRestPose) -> Option<Vec<VisualTransform>> {
    description
      .bones
      .iter()
      .zip(&pose.transforms)
      .map(|(bone, posed)| bone.bind_transform.as_ref().map(|bind| compose(posed, &invert(bind))))
      .collect()
  }

  fn pose_geometry(
    geometry: &VisualGeometry,
    buffer: &[u8],
    skins: Option<&[VisualTransform]>,
    builder: &mut VisualBufferBuilder,
  ) -> VisualGeometry {
    let mut positions: Vec<f32> = read_f32s(buffer, geometry.positions);
    let mut normals: Vec<f32> = read_f32s(buffer, geometry.normals);
    let mut tangents: Vec<f32> = read_f32s(buffer, geometry.tangents);
    let mut binormals: Vec<f32> = read_f32s(buffer, geometry.binormals);
    let uvs: Vec<f32> = read_f32s(buffer, geometry.uvs);
    let indices: Vec<u16> = read_u16s(buffer, geometry.indices);

    if let (Some(skin), Some(skins)) = (&geometry.skin, skins) {
      let links: Vec<u16> = read_u16s(buffer, skin.indices);
      let weights: Vec<f32> = read_f32s(buffer, skin.weights);

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

        move_point(&mut positions, vertex, &linked);

        for directions in [&mut normals, &mut tangents, &mut binormals] {
          turn_direction(directions, vertex, &linked);
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
}

/// A point moved by each bone it hangs from, weighted, in place.
fn move_point(values: &mut [f32], vertex: usize, linked: &[(&VisualTransform, f32)]) {
  let point: [f32; 3] = [values[vertex * 3], values[vertex * 3 + 1], values[vertex * 3 + 2]];
  let mut moved: [f32; 3] = [0.0; 3];

  for (skin, weight) in linked.iter().copied() {
    let turned: [f32; 3] = rotate(skin, point);

    moved[0] += (turned[0] + skin.c.x) * weight;
    moved[1] += (turned[1] + skin.c.y) * weight;
    moved[2] += (turned[2] + skin.c.z) * weight;
  }

  values[vertex * 3..vertex * 3 + 3].copy_from_slice(&moved);
}

/// A direction turned by each bone it hangs from, weighted, and made a unit again, in place; one that sums to nothing
/// stays as it was.
fn turn_direction(values: &mut [f32], vertex: usize, linked: &[(&VisualTransform, f32)]) {
  if values.len() < vertex * 3 + 3 {
    return;
  }

  let direction: [f32; 3] = [values[vertex * 3], values[vertex * 3 + 1], values[vertex * 3 + 2]];
  let mut turned: [f32; 3] = [0.0; 3];

  for (skin, weight) in linked.iter().copied() {
    let by: [f32; 3] = rotate(skin, direction);

    turned[0] += by[0] * weight;
    turned[1] += by[1] * weight;
    turned[2] += by[2] * weight;
  }

  let length: f32 = (turned[0] * turned[0] + turned[1] * turned[1] + turned[2] * turned[2]).sqrt();

  if length > 0.0 {
    values[vertex * 3..vertex * 3 + 3].copy_from_slice(&turned.map(|it| it / length));
  }
}

/// A vector turned by a transform's basis, its translation left out.
fn rotate(transform: &VisualTransform, [x, y, z]: [f32; 3]) -> [f32; 3] {
  let VisualTransform { i, j, k, .. } = transform;

  [
    i.x * x + j.x * y + k.x * z,
    i.y * x + j.y * y + k.y * z,
    i.z * x + j.z * y + k.z * z,
  ]
}

/// `outer` after `inner`: a point taken by `inner`, then by `outer`.
fn compose(outer: &VisualTransform, inner: &VisualTransform) -> VisualTransform {
  let column = |vector: &Vector3d| {
    let [x, y, z] = rotate(outer, [vector.x, vector.y, vector.z]);

    Vector3d::new(x, y, z)
  };
  let [x, y, z] = rotate(outer, [inner.c.x, inner.c.y, inner.c.z]);

  VisualTransform {
    i: column(&inner.i),
    j: column(&inner.j),
    k: column(&inner.k),
    c: Vector3d::new(x + outer.c.x, y + outer.c.y, z + outer.c.z),
  }
}

/// The inverse of a rigid transform: its basis transposed, and its translation turned back by it.
fn invert(transform: &VisualTransform) -> VisualTransform {
  let VisualTransform { i, j, k, c } = transform;
  let transposed: VisualTransform = VisualTransform {
    i: Vector3d::new(i.x, j.x, k.x),
    j: Vector3d::new(i.y, j.y, k.y),
    k: Vector3d::new(i.z, j.z, k.z),
    c: Vector3d::new(0.0, 0.0, 0.0),
  };
  let [x, y, z] = rotate(&transposed, [c.x, c.y, c.z]);

  VisualTransform {
    c: Vector3d::new(-x, -y, -z),
    ..transposed
  }
}

fn read_f32s(buffer: &[u8], section: VisualSection) -> Vec<f32> {
  get_bytes(buffer, section)
    .as_chunks::<4>()
    .0
    .iter()
    .map(|bytes| f32::from_le_bytes(*bytes))
    .collect()
}

fn read_u16s(buffer: &[u8], section: VisualSection) -> Vec<u16> {
  get_bytes(buffer, section)
    .as_chunks::<2>()
    .0
    .iter()
    .map(|bytes| u16::from_le_bytes(*bytes))
    .collect()
}

/// A section's bytes, shortened where the buffer ends before it does.
fn get_bytes(buffer: &[u8], section: VisualSection) -> &[u8] {
  let start: usize = (section.byte_offset as usize).min(buffer.len());
  let end: usize = (start + section.byte_length as usize).min(buffer.len());

  &buffer[start..end]
}
