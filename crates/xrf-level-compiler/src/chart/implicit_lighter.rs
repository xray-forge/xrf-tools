use glam::Vec3;
use rayon::prelude::*;
use xrf_geometry::Tracer;
use xrf_math::{Vector2d, Vector3d};

use crate::chart::implicit_map::ImplicitMap;
use crate::chart::light_chart_lighter::LightChartLighter;
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

  /// The map of texture `name`, of the texels `base` holds, over `faces` (mesh faces of it, in the mesh's order). A
  /// texel sums the light of every face under each of its jitter samples (`ImplicitExecute::Execute`), a face lit at its
  /// place there facing its corners' normals blended there and normalised, the face itself let through, and takes the
  /// mean halved; the texels unlit are then filled ring by ring to the map's edges. The texture's alpha becomes each
  /// texel's hemisphere, and the map its static light and sun.
  pub fn light(&self, name: String, mut base: Vec<[u8; 4]>, width: u32, height: u32, faces: &[u32]) -> ImplicitMap {
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
    let rows: Vec<Vec<(LightColor, u8)>> = (0..height)
      .into_par_iter()
      .map(|y| {
        let mut occluders: LightOccluders = LightOccluders::new(self.lighter.lights);

        (0..width)
          .map(|x| {
            let cell: &[u32] = &cells[((y / Self::CELL) * columns + x / Self::CELL) as usize];
            let mut sum: LightColor = LightColor::default();
            let mut count: u32 = 0;

            for [jitter_x, jitter_y] in jitter {
              let point: Vector2d = Vector2d::new(
                x as f32 / dimension_x + half_x + jitter_x * spread_x,
                y as f32 / dimension_y + half_y + jitter_y * spread_y,
              );

              for face in cell {
                let Some(weights) = LightChartLighter::<T>::get_barycentric(&self.coordinates[*face as usize], &point)
                else {
                  continue;
                };
                let color: LightColor = self.sample_face(*face, &weights, &mut occluders);

                sum = LightColor {
                  rgb: sum.rgb + color.rgb,
                  hemi: sum.hemi + color.hemi,
                  sun: sum.sun + color.sun,
                };
                count += 1;
              }
            }

            if count > 0 {
              (
                sum.get_scaled(1.0 / count as f32).get_scaled(0.5).get_stored(),
                LightmapLayer::LIT,
              )
            } else {
              (LightColor::default(), 0)
            }
          })
          .collect()
      })
      .collect();
    let mut layer: LightmapLayer = LightmapLayer::new(width, height);

    for (index, (color, marker)) in rows.into_iter().flatten().enumerate() {
      layer.surface[index] = color;
      layer.marker[index] = marker;
    }

    layer.fill_borders(254);

    for (texel, color) in base.iter_mut().zip(&layer.surface) {
      texel[3] = LightColor::get_byte(color.hemi);
    }

    ImplicitMap {
      name,
      width,
      height,
      base,
      lightmap: layer.get_packed(),
    }
  }

  /// The light at `weights` on `face`: its corners blended there, facing its corners' normals blended there and
  /// normalised (`Fvector::normalize`).
  fn sample_face(&self, face: u32, weights: &[f32; 3], occluders: &mut LightOccluders) -> LightColor {
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

    self.lighter.light_point(
      face,
      Vec3::new(point.x, point.y, point.z),
      Vec3::new(normal.x * scale, normal.y * scale, normal.z * scale),
      occluders,
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
