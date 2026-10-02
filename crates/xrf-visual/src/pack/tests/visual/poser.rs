use xrf_ogf::{OgfFile, OgfIkDataChunk};

use crate::data::visual::geometry::visual_geometry::VisualGeometry;
use crate::data::visual::skeleton::visual_rest_pose::VisualRestPose;
use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::pack::tests::fixtures::{bind, bones, skeleton, static_triangle_child, vector};
use crate::pack::visual::visual_package::VisualPackage;
use crate::pack::visual::visual_packer::VisualPacker;
use crate::pack::visual::visual_poser::VisualPoser;

/// A triangle hanging from the first of two bones, both bound where the model stands.
fn skinned_triangle() -> VisualPackage {
  let file: OgfFile = OgfFile {
    bones: Some(bones(&[("root", ""), ("child", "root")])),
    ik_data: Some(OgfIkDataChunk {
      bones: vec![
        bind(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 0.0)),
        bind(vector(0.0, 0.0, 0.0), vector(0.0, 1.0, 0.0)),
      ],
    }),
    ..skeleton(vec![static_triangle_child()])
  };

  VisualPacker::pack(&file)
}

fn geometry(package: &VisualPackage) -> &VisualGeometry {
  package.description.submeshes[0]
    .geometry()
    .expect("the triangle to pack")
}

fn positions(package: &VisualPackage) -> Vec<f32> {
  let section = geometry(package).positions;

  package.buffer[section.byte_offset as usize..(section.byte_offset + section.byte_length) as usize]
    .as_chunks::<4>()
    .0
    .iter()
    .map(|bytes| f32::from_le_bytes(*bytes))
    .collect()
}

/// The two bones posed: the first raised two metres, the second where it is bound.
fn raised() -> VisualRestPose {
  let raised: VisualTransform = VisualTransform {
    i: vector(1.0, 0.0, 0.0),
    j: vector(0.0, 1.0, 0.0),
    k: vector(0.0, 0.0, 1.0),
    c: vector(0.0, 2.0, 0.0),
  };
  let child: VisualTransform = VisualTransform {
    c: vector(0.0, 1.0, 0.0),
    ..raised.clone()
  };

  VisualRestPose::new(vec![String::from("root"), String::from("child")], vec![raised, child])
    .expect("two transforms for two bones")
}

#[test]
fn moves_a_skinned_vertex_with_its_bone_and_cuts_the_posed_geometry_into_clusters() {
  let package: VisualPackage = skinned_triangle();

  assert!(geometry(&package).skin.is_some());
  assert!(geometry(&package).clusters.is_none());

  let posed: VisualPackage = VisualPoser::pose(&package, Some(&raised()));
  let before: Vec<f32> = positions(&package);
  let after: Vec<f32> = positions(&posed);

  assert!(geometry(&posed).skin.is_none());
  assert!(geometry(&posed).clusters.is_some());
  assert_eq!(after.len(), before.len());

  for (moved, stored) in after.chunks(3).zip(before.chunks(3)) {
    assert!((moved[0] - stored[0]).abs() < 1e-6);
    assert!((moved[1] - stored[1] - 2.0).abs() < 1e-6);
    assert!((moved[2] - stored[2]).abs() < 1e-6);
  }
}

#[test]
fn stands_a_visual_without_a_pose_as_stored() {
  let package: VisualPackage = skinned_triangle();
  let posed: VisualPackage = VisualPoser::pose(&package, None);

  assert_eq!(positions(&posed), positions(&package));
  assert_eq!(posed.description.buffer_length as usize, posed.buffer.len());
}
