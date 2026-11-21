use std::ops::Range;

use glam::Vec3;
use xrf_error::XrfResult;
use xrf_job::parallel::prelude::*;
use xrf_job::{JobHandle, JobScope};
use xrf_math::{Vector2d, Vector3d};

use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::lit::light_chart_lighter::LightChartLighter;
use crate::light::lightmap::lit::light_texel_point::LightTexelPoint;
use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_occluders::LightOccluders;

/// What lights a texture xrLC lights implicitly, such as terrain's (`ImplicitLightingExec`,
/// `xrLC_Light/xrLight_Implicit.cpp`): a map the texture's size laid over its own coordinates, each texel lit where its
/// faces cover it, with [`LightChartLighter`]'s light.
pub(crate) struct ImplicitLighter<'a> {
  pub(crate) lighter: &'a LightChartLighter<'a>,
  /// Each mesh face's base texture coordinates, as the tangent-basis pass leaves them.
  pub(crate) coordinates: &'a [[Vector2d; 3]],
  /// The job a map's rows are counted to as each band is lit, and which stops the lighting when cancelled.
  pub(crate) job: &'a JobHandle,
}

impl ImplicitLighter<'_> {
  /// What a job reports a map's lighting under: its rows.
  pub(crate) const PROGRESS_ROWS: &'static str = "implicit map rows";

  /// The texels a side of a cell of the grid the faces are found by.
  const CELL: u32 = 8;

  /// About how many points a band of the map is lit with at once, as many as the GPU lights charts in at a time; a
  /// whole terrain map's points would hold gigabytes.
  const BAND_POINTS: usize = 1 << 21;

  /// The map of texture `name`, of the texels `base` holds, over its mesh faces `faces` in the mesh's order, lit on the
  /// CPU a row's worth of points at a time, each light first tested against the face that last stopped it.
  ///
  /// # Errors
  ///
  /// Returns an error when the job is cancelled.
  pub(crate) fn light(
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
  /// Returns an error when the GPU refuses a batch, or the job is cancelled.
  #[cfg(feature = "gpu")]
  pub(crate) fn light_on_gpu(
    &self,
    gpu: &crate::light::lightmap::lightmap_gpu::LightmapGpu<'_>,
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

  /// The map of texture `name`, of the texels `base` holds, over `faces`, its points lit by `light_points` a band of
  /// rows at a time ([`Self::BAND_POINTS`]).
  ///
  /// A texel takes half the mean light of every face under each of its jitter samples (`ImplicitExecute::Execute`),
  /// each facing its corners' normals blended there, the face itself let through. Unlit texels are then filled ring by
  /// ring to the map's edges. Each texel's hemisphere becomes the texture's alpha; the map holds static light and sun.
  fn light_by(
    &self,
    name: String,
    mut base: Vec<[u8; 4]>,
    width: u32,
    height: u32,
    faces: &[u32],
    mut light_points: impl FnMut(&[LightTexelPoint]) -> XrfResult<Vec<LightColor>>,
  ) -> XrfResult<ImplicitMap> {
    let jitter: &[[f32; 2]] = self.lighter.scene.get_jitter();
    let (dimension_x, dimension_y) = (width as f32, height as f32);
    let (half_x, half_y) = (0.5 / dimension_x, 0.5 / dimension_y);
    let (spread_x, spread_y) = (0.499 / dimension_x, 0.499 / dimension_y);
    let cells: Vec<Vec<u32>> = self.get_cells(faces, width, height);
    let columns: u32 = width.div_ceil(Self::CELL);
    let mut layer: LightmapLayer = LightmapLayer::new(width, height);
    let band: u32 = Self::get_band_rows(width, height, jitter.len());
    let scope: JobScope = self.job.enter(Self::PROGRESS_ROWS, Some(u64::from(height)));

    for first in (0..height).step_by(band as usize) {
      self.job.check_cancelled()?;

      let rows: Range<u32> = first..(first + band).min(height);
      let row_count: u64 = u64::from(rows.end - rows.start);
      let texels: Range<usize> = (rows.start * width) as usize..(rows.end * width) as usize;
      let points: Vec<LightTexelPoint> = rows
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
                if let Some(weights) = LightChartLighter::get_barycentric(&self.coordinates[*face as usize], &sample) {
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

      LightChartLighter::assemble_into(&mut layer, texels, &points, &colors);
      scope.advance_by(row_count);
    }

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

  /// How many rows of a map `width` by `height` a band holds at `samples` jitter samples a texel: at least one, and as
  /// many as keep its points near [`Self::BAND_POINTS`] where each sample falls on one face, as on terrain. A texel's
  /// samples stay in its row, so a band lights on its own.
  fn get_band_rows(width: u32, height: u32, samples: usize) -> u32 {
    let row_points: usize = (width as usize * samples).max(1);

    (Self::BAND_POINTS / row_points).clamp(1, height.max(1) as usize) as u32
  }

  /// The point at `weights` on `face`: its corners blended there, facing its corners' normals blended there and
  /// normalised (`Fvector::normalize`).
  fn get_sample(&self, face: u32, weights: &[f32; 3]) -> (Vec3, Vec3) {
    let normals: &GeometryNormals = self.lighter.scene.normals;
    let splits: [u32; 3] = normals.faces[face as usize];
    let point: Vector3d = LightChartLighter::blend(splits.map(|split| normals.positions[split as usize]), weights);
    let normal: Vector3d = LightChartLighter::blend(splits.map(|split| normals.normals[split as usize]), weights);
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
