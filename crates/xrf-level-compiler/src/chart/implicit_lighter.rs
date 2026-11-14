use glam::Vec3;
use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_geometry::Tracer;
use xrf_math::{Vector2d, Vector3d};

use crate::chart::implicit_map::ImplicitMap;
use crate::chart::light_chart_lighter::LightChartLighter;
use crate::chart::light_texel_point::LightTexelPoint;
use crate::chart::lightmap_layer::LightmapLayer;
use crate::geometry::geometry_normals::GeometryNormals;
use crate::light::light_color::LightColor;
use crate::light::light_occluders::LightOccluders;

/// What lights a texture xrLC lights implicitly, such as terrain's (`ImplicitLightingExec`,
/// `xrLC_Light/xrLight_Implicit.cpp`): a map the texture's size laid over its own coordinates, each texel lit where its
/// faces cover it, with [`LightChartLighter`]'s light.
pub struct ImplicitLighter<'a, T: Tracer> {
  pub lighter: &'a LightChartLighter<'a, T>,
  /// Each mesh face's base texture coordinates, as the tangent-basis pass leaves them.
  pub coordinates: &'a [[Vector2d; 3]],
}

impl<T: Tracer + Sync> ImplicitLighter<'_, T> {
  /// The texels a side of a cell of the grid the faces are found by.
  const CELL: u32 = 8;

  /// `Jitter_Select`'s offsets, in half texels.
  const JITTER_1: [[f32; 2]; 1] = [[0.0, 0.0]];
  const JITTER_4: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
  const JITTER_9: [[f32; 2]; 9] = [
    [-1.0, -1.0],
    [0.0, -1.0],
    [1.0, -1.0],
    [-1.0, 0.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [-1.0, 1.0],
    [0.0, 1.0],
    [1.0, 1.0],
  ];

  /// The map of texture `name`, of the texels `base` holds, over `faces` (mesh faces of it, in the mesh's order), its
  /// points lit on the CPU a row's worth at a time, each light first tested against the face that last stopped it.
  ///
  /// # Errors
  ///
  /// Returns no error on the CPU; the result is a [`Self::light_by`]'s.
  pub fn light(
    &self,
    name: String,
    base: Vec<[u8; 4]>,
    width: u32,
    height: u32,
    faces: &[u32],
  ) -> XrfResult<ImplicitMap> {
    self.light_by(name, base, width, height, faces, |points| {
      Ok(
        points
          .par_chunks(width.max(1) as usize)
          .map(|chunk| {
            let mut occluders: LightOccluders = LightOccluders::new(self.lighter.lights);

            chunk
              .iter()
              .map(|point| {
                self
                  .lighter
                  .light_point(point.face, point.point, point.normal, &mut occluders)
              })
              .collect::<Vec<LightColor>>()
          })
          .collect::<Vec<_>>()
          .into_iter()
          .flatten()
          .collect(),
      )
    })
  }

  /// [`Self::light`] with the points' shadow rays traced on `gpu`.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch.
  #[cfg(feature = "gpu")]
  pub(crate) fn light_on_gpu(
    &self,
    gpu: &crate::chart::light_gpu::LightGpu,
    name: String,
    base: Vec<[u8; 4]>,
    width: u32,
    height: u32,
    faces: &[u32],
  ) -> XrfResult<ImplicitMap> {
    self.light_by(name, base, width, height, faces, |points| {
      gpu.light_points(self.lighter.lights, self.lighter.ray_faces, points)
    })
  }

  /// The map of texture `name`, of the texels `base` holds, over `faces`, its points lit by `light_points`. A texel sums
  /// the light of every face under each of its jitter samples (`ImplicitExecute::Execute`), a face lit at its place
  /// there facing its corners' normals blended there and normalised, the face itself let through, and takes the mean
  /// halved; the texels unlit are then filled ring by ring to the map's edges. The texture's alpha becomes each
  /// texel's hemisphere, and the map its static light and sun.
  fn light_by(
    &self,
    name: String,
    mut base: Vec<[u8; 4]>,
    width: u32,
    height: u32,
    faces: &[u32],
    light_points: impl FnOnce(&[LightTexelPoint]) -> XrfResult<Vec<LightColor>>,
  ) -> XrfResult<ImplicitMap> {
    let jitter: &[[f32; 2]] = match self.lighter.scene.jitter_samples {
      1 => &Self::JITTER_1,
      9 => &Self::JITTER_9,
      _ => &Self::JITTER_4,
    };
    let (dimension_x, dimension_y) = (width as f32, height as f32);
    let (half_x, half_y) = (0.5 / dimension_x, 0.5 / dimension_y);
    let (spread_x, spread_y) = (0.499 / dimension_x, 0.499 / dimension_y);
    let cells: Vec<Vec<u32>> = self.get_cells(faces, width, height);
    let columns: u32 = width.div_ceil(Self::CELL);
    let points: Vec<LightTexelPoint> = (0..height)
      .into_par_iter()
      .map(|y| {
        let mut row: Vec<LightTexelPoint> = Vec::new();

        for x in 0..width {
          let cell: &[u32] = &cells[((y / Self::CELL) * columns + x / Self::CELL) as usize];

          for [jitter_x, jitter_y] in jitter {
            let sample: Vector2d = Vector2d::new(
              x as f32 / dimension_x + half_x + jitter_x * spread_x,
              y as f32 / dimension_y + half_y + jitter_y * spread_y,
            );

            for face in cell {
              if let Some(weights) = LightChartLighter::<T>::get_barycentric(&self.coordinates[*face as usize], &sample)
              {
                let (point, normal) = self.get_sample(*face, &weights);

                row.push(LightTexelPoint {
                  texel: y * width + x,
                  face: *face,
                  point,
                  normal,
                  is_edge: false,
                });
              }
            }
          }
        }

        row
      })
      .collect::<Vec<_>>()
      .into_iter()
      .flatten()
      .collect();
    let colors: Vec<LightColor> = light_points(&points)?;
    let mut layer: LightmapLayer = LightChartLighter::<T>::assemble(width, height, &points, &colors);

    layer.fill_borders(254);

    for (texel, color) in base.iter_mut().zip(&layer.surface) {
      texel[3] = LightColor::get_byte(color.hemi);
    }

    Ok(ImplicitMap {
      name,
      width,
      height,
      base,
      lightmap: layer.get_packed(),
    })
  }

  /// The point at `weights` on `face`: its corners blended there, facing its corners' normals blended there and
  /// normalised (`Fvector::normalize`).
  fn get_sample(&self, face: u32, weights: &[f32; 3]) -> (Vec3, Vec3) {
    let normals: &GeometryNormals = self.lighter.scene.normals;
    let splits: [u32; 3] = normals.faces[face as usize];
    let blend = |values: [Vector3d; 3]| {
      Vector3d::new(
        values[0].x * weights[0] + values[1].x * weights[1] + values[2].x * weights[2],
        values[0].y * weights[0] + values[1].y * weights[1] + values[2].y * weights[2],
        values[0].z * weights[0] + values[1].z * weights[1] + values[2].z * weights[2],
      )
    };
    let point: Vector3d = blend(splits.map(|split| normals.positions[split as usize]));
    let normal: Vector3d = blend(splits.map(|split| normals.normals[split as usize]));
    let scale: f32 = (1.0 / (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z)).sqrt();

    (
      Vec3::new(point.x, point.y, point.z),
      Vec3::new(normal.x * scale, normal.y * scale, normal.z * scale),
    )
  }

  /// Each cell of [`Self::CELL`] texels a side, row by row over a map `width` by `height`: the faces whose coordinates
  /// may hold one of its texels' samples, in their order (`IHASH`).
  fn get_cells(&self, faces: &[u32], width: u32, height: u32) -> Vec<Vec<u32>> {
    let (columns, rows) = (width.div_ceil(Self::CELL), height.div_ceil(Self::CELL));
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); (columns * rows) as usize];
    let span = |values: [f32; 3], side: u32| {
      let low: f32 = values.into_iter().fold(f32::INFINITY, f32::min);
      let high: f32 = values.into_iter().fold(f32::NEG_INFINITY, f32::max);
      let first: i64 = ((low * side as f32).floor() as i64 - 1).max(0);
      let last: i64 = ((high * side as f32).ceil() as i64 + 1).min(i64::from(side) - 1);

      (first <= last).then_some((first as u32 / Self::CELL, last as u32 / Self::CELL))
    };

    for face in faces {
      let coordinates: &[Vector2d; 3] = &self.coordinates[*face as usize];
      let (Some((first_x, last_x)), Some((first_y, last_y))) = (
        span(coordinates.map(|coordinate| coordinate.x), width),
        span(coordinates.map(|coordinate| coordinate.y), height),
      ) else {
        continue;
      };

      for y in first_y..=last_y {
        for x in first_x..=last_x {
          cells[(y * columns + x) as usize].push(*face);
        }
      }
    }

    cells
  }
}
