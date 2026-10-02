use crate::cform::level_cform_tracer_node::LevelCformTracerNode;

/// A box a metre across from the origin.
fn unit_box() -> LevelCformTracerNode {
  let mut node: LevelCformTracerNode = LevelCformTracerNode::leaf(0, 0);

  node.enclose(&[0.0, 0.0, 0.0], &[1.0, 1.0, 1.0]);

  node
}

fn inverse(direction: [f32; 3]) -> [f32; 3] {
  direction.map(|it| 1.0 / it)
}

#[test]
fn reaches_a_box_along_its_axes_and_misses_one_beside_them() {
  let node: LevelCformTracerNode = unit_box();

  assert!(node.is_reached(&[0.5, 0.5, -1.0], &inverse([0.0, 0.0, 1.0]), 5.0));
  assert!(!node.is_reached(&[0.5, 0.5, -1.0], &inverse([0.0, 0.0, 1.0]), 0.5));
  assert!(!node.is_reached(&[1.5, 0.5, -1.0], &inverse([0.0, 0.0, 1.0]), 5.0));
  assert!(!node.is_reached(&[0.5, 0.5, -1.0], &inverse([0.0, 0.0, -1.0]), 5.0));
}

// A sky direction with a zero component, from an origin on a face's plane, ran along the face and was refused.
#[test]
fn reaches_a_box_along_a_face_from_an_origin_on_its_plane() {
  let node: LevelCformTracerNode = unit_box();

  assert!(node.is_reached(&[0.0, 0.5, -1.0], &inverse([0.0, 0.0, 1.0]), 5.0));
  assert!(node.is_reached(&[1.0, 0.5, -1.0], &inverse([0.0, 0.0, 1.0]), 5.0));
  assert!(!node.is_reached(&[1.0001, 0.5, -1.0], &inverse([0.0, 0.0, 1.0]), 5.0));
}
