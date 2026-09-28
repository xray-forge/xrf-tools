use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_file::LevelCformHeader;
use crate::cform::level_cform_geometry::LevelCformGeometry;

/// What of the collision form stands over a set of points, read one face at a time rather than keeping the form.
pub(crate) struct LevelCformCover;

impl LevelCformCover {
  /// Whether no face of the payload after a header lies straight above each point, which is under the open sky.
  pub(crate) fn read_open_above<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    header: &LevelCformHeader,
    points: &[Vector3d<f32>],
  ) -> XrfResult<Vec<bool>> {
    let mut vertices: Vec<Vector3d<f32>> = reader.new_bounded_vec(
      u64::from(header.vertex_count),
      LevelCformGeometry::VERTEX_SIZE as u64,
      "collision form vertices",
    )?;

    for _ in 0..header.vertex_count {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    let mut open: Vec<bool> = vec![true; points.len()];

    for _ in 0..header.face_count {
      let face: LevelCformFace = reader.read_xr::<T, _>()?;
      let [a, b, c]: [&Vector3d<f32>; 3] = match face.vertices.map(|index| vertices.get(index as usize)) {
        [Some(a), Some(b), Some(c)] => [a, b, c],
        _ => {
          return Err(XrfError::new_invalid_error(format!(
            "A collision form face names vertex {:?} of {}",
            face.vertices, header.vertex_count
          )));
        }
      };

      for (point, is_open) in points.iter().zip(open.iter_mut()) {
        if *is_open && Self::is_triangle_above(a, b, c, point) {
          *is_open = false;
        }
      }
    }

    Ok(open)
  }

  /// Whether a vertical line through a point crosses a triangle above it: a face seen edge-on from above covers nothing.
  fn is_triangle_above(a: &Vector3d<f32>, b: &Vector3d<f32>, c: &Vector3d<f32>, point: &Vector3d<f32>) -> bool {
    if point.x < a.x.min(b.x).min(c.x)
      || point.x > a.x.max(b.x).max(c.x)
      || point.z < a.z.min(b.z).min(c.z)
      || point.z > a.z.max(b.z).max(c.z)
      || point.y > a.y.max(b.y).max(c.y)
    {
      return false;
    }

    let area: f32 = (b.x - a.x) * (c.z - a.z) - (c.x - a.x) * (b.z - a.z);

    if area.abs() <= f32::EPSILON {
      return false;
    }

    let u: f32 = ((b.x - point.x) * (c.z - point.z) - (c.x - point.x) * (b.z - point.z)) / area;
    let v: f32 = ((c.x - point.x) * (a.z - point.z) - (a.x - point.x) * (c.z - point.z)) / area;
    let w: f32 = 1.0 - u - v;

    u >= 0.0 && v >= 0.0 && w >= 0.0 && u * a.y + v * b.y + w * c.y > point.y
  }
}
