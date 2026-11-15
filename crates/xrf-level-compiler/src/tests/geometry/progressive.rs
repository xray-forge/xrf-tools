//! How a visual is made progressive, `OGF::MakeProgressive`.

use std::collections::HashSet;

use xrf_math::{Vector2d, Vector3d};

use crate::geometry::surface::geometry_basis::GeometryBasis;
use crate::geometry::visual::geometry_fast_path::GeometryFastPath;
use crate::geometry::visual::geometry_visual::GeometryVisual;
use crate::geometry::visual::geometry_visual_light::GeometryVisualLight;
use crate::geometry::visual::geometry_visual_vertex::GeometryVisualVertex;
use crate::light::point::light_color::LightColor;

/// A flat opaque visual of a grid `side` points a side, a unit apart, its base coordinates following its positions.
fn grid_visual(side: u16) -> GeometryVisual {
  let mut visual: GeometryVisual = GeometryVisual::new(
    0,
    0,
    vec![String::from("floor")],
    String::from("default/floor"),
    GeometryVisualLight::Vertex,
  );
  let corner = |x: u16, z: u16| GeometryVisualVertex {
    position: Vector3d::new(f32::from(x), 0.0, f32::from(z)),
    normal: Vector3d::new(0.0, 1.0, 0.0),
    basis: GeometryBasis::NONE,
    texture_coordinate: Vector2d::new(f32::from(x) / 4.0, f32::from(z) / 4.0),
    lightmap_coordinate: None,
    light: LightColor::default(),
  };
  let texel: [Vector2d; 1] = [Vector2d::new(1.0 / 512.0, 1.0 / 512.0)];

  for z in 0..side - 1 {
    for x in 0..side - 1 {
      visual.add_face(
        [corner(x, z), corner(x, z + 1), corner(x + 1, z)],
        &texel,
        GeometryFastPath::Joined,
      );
      visual.add_face(
        [corner(x + 1, z), corner(x, z + 1), corner(x + 1, z + 1)],
        &texel,
        GeometryFastPath::Joined,
      );
    }
  }

  visual
}

/// A visual's triangles by their corners' positions, as bits.
fn positions(vertices: &[Vector3d], faces: &[[u16; 3]]) -> HashSet<[[u32; 3]; 3]> {
  faces
    .iter()
    .map(|face| {
      let mut corners: [[u32; 3]; 3] = face.map(|vertex| {
        let position: Vector3d = vertices[usize::from(vertex)];

        [position.x.to_bits(), position.y.to_bits(), position.z.to_bits()]
      });

      corners.sort_unstable();
      corners
    })
    .collect()
}

#[test]
fn makes_a_large_visual_progressive_keeping_its_full_detail() {
  let mut visual: GeometryVisual = grid_visual(10);
  let before: HashSet<[[u32; 3]; 3]> = positions(
    &visual.vertices.iter().map(|vertex| vertex.position).collect::<Vec<_>>(),
    &visual.faces,
  );
  let fast_before: HashSet<[[u32; 3]; 3]> = positions(&visual.fast_vertices, &visual.fast_faces);
  let triangles: usize = visual.faces.len();

  assert!(triangles >= GeometryVisual::PROGRESSIVE_FACE_LIMIT);

  visual.make_progressive(GeometryVisual::STATIC_METRIC_LIMIT);

  let windows = visual.windows.clone().expect("a large flat visual is made progressive");
  let fast_windows = visual.fast_windows.clone().expect("its fast path too");
  let full = |faces: &[[u16; 3]], window: &xrf_geometry::ProgressiveWindow| {
    let start: usize = window.offset as usize / 3;

    faces[start..start + usize::from(window.triangles)].to_vec()
  };

  assert_eq!(visual.get_triangle_count(), triangles);
  assert_eq!(
    positions(
      &visual.vertices.iter().map(|vertex| vertex.position).collect::<Vec<_>>(),
      &full(&visual.faces, &windows[0]),
    ),
    before,
    "the finest window draws the visual as it was"
  );
  assert_eq!(
    positions(&visual.fast_vertices, &full(&visual.fast_faces, &fast_windows[0])),
    fast_before
  );
  assert!(windows.last().expect("windows").triangles < windows[0].triangles);
}

#[test]
fn leaves_a_small_visual_whole() {
  let mut visual: GeometryVisual = grid_visual(5);
  let faces: Vec<[u16; 3]> = visual.faces.clone();

  visual.make_progressive(GeometryVisual::STATIC_METRIC_LIMIT);

  assert!(visual.windows.is_none() && visual.fast_windows.is_none());
  assert_eq!(visual.faces, faces);
}

#[test]
fn leaves_a_visual_whole_whose_windows_gain_too_little() {
  let mut visual: GeometryVisual = grid_visual(10);
  let faces: Vec<[u16; 3]> = visual.faces.clone();

  visual.make_progressive(1.0);

  assert!(visual.windows.is_none(), "no visual drops all its vertices");
  assert_eq!(visual.faces, faces);
}
