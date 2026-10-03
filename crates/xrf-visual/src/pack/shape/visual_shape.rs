use std::f32::consts::PI;

use crate::pack::visual::visual_mesh::VisualMesh;

/// Which axis a box face runs across, which down, and which it faces along; the signs it runs with; and its width,
/// height and the distance to it, signed for the side it faces.
type BoxFace = ([usize; 3], [f32; 2], [f32; 3]);

/// Bodies built rather than read, which a texture is laid on to be looked at.
pub struct VisualShape;

impl VisualShape {
  /// A box about the origin, a mesh a face in the order `+x`, `-x`, `+y`, `-y`, `+z`, `-z`, each face's uvs running
  /// from its top left corner as X-Ray stores rows.
  pub fn create_box(width: f32, height: f32, depth: f32) -> [VisualMesh; 6] {
    let faces: [BoxFace; 6] = [
      ([2, 1, 0], [-1.0, -1.0], [depth, height, width]),
      ([2, 1, 0], [1.0, -1.0], [depth, height, -width]),
      ([0, 2, 1], [1.0, 1.0], [width, depth, height]),
      ([0, 2, 1], [1.0, -1.0], [width, depth, -height]),
      ([0, 1, 2], [1.0, -1.0], [width, height, depth]),
      ([0, 1, 2], [-1.0, -1.0], [width, height, -depth]),
    ];

    faces.map(
      |([across, down, out], [across_sign, down_sign], [face_width, face_height, face_depth])| {
        let mut mesh: VisualMesh = VisualMesh::default();
        let mut facing: [f32; 3] = [0.0; 3];

        facing[out] = face_depth.signum();

        for row in 0..2 {
          for column in 0..2 {
            let mut vertex: [f32; 3] = [0.0; 3];

            vertex[across] = (column as f32 - 0.5) * face_width * across_sign;
            vertex[down] = (row as f32 - 0.5) * face_height * down_sign;
            vertex[out] = face_depth / 2.0;

            mesh.positions.push(vertex);
            mesh.normals.push(facing);
            mesh.uvs.push([column as f32, row as f32]);
          }
        }

        mesh.indices = vec![0, 2, 1, 2, 3, 1];
        mesh
      },
    )
  }

  /// A sphere about the origin, `width_segments` around and `height_segments` from pole to pole, its uvs wrapping
  /// once around and once down.
  pub fn create_sphere(radius: f32, width_segments: u16, height_segments: u16) -> VisualMesh {
    let mut mesh: VisualMesh = VisualMesh::default();
    let mut grid: Vec<Vec<u16>> = Vec::with_capacity(usize::from(height_segments) + 1);
    let width: f32 = f32::from(width_segments);

    for row in 0..=height_segments {
      let v: f32 = f32::from(row) / f32::from(height_segments);
      // The poles' uvs shift half a segment, so each pole triangle samples the middle of its own column.
      let offset: f32 = match row {
        0 => 0.5 / width,
        _ if row == height_segments => -0.5 / width,
        _ => 0.0,
      };

      grid.push(
        (0..=width_segments)
          .map(|column| {
            let u: f32 = f32::from(column) / width;
            let normal: [f32; 3] = [
              -(u * PI * 2.0).cos() * (v * PI).sin(),
              (v * PI).cos(),
              (u * PI * 2.0).sin() * (v * PI).sin(),
            ];
            let index: u16 = mesh.positions.len() as u16;

            mesh.positions.push(normal.map(|it| it * radius));
            mesh.normals.push(normal);
            mesh.uvs.push([u + offset, v]);

            index
          })
          .collect(),
      );
    }

    for row in 0..usize::from(height_segments) {
      for column in 0..usize::from(width_segments) {
        let (a, b, c, d) = (
          grid[row][column + 1],
          grid[row][column],
          grid[row + 1][column],
          grid[row + 1][column + 1],
        );

        if row != 0 {
          mesh.indices.extend([a, b, d]);
        }

        if row != usize::from(height_segments) - 1 {
          mesh.indices.extend([b, c, d]);
        }
      }
    }

    mesh
  }
}
