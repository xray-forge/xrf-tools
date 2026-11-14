use glam::Vec3;
use xrf_geometry::TraceHit;
use xrf_level::LevelLightRole;
use xrf_math::{Vector2d, Vector3d};

use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::lightmap::chart::light_chart_scene::LightChartScene;
use crate::light::lightmap::chart::lightmap_orientation::LightmapOrientation;
use crate::light::lightmap::lit::light_texel_point::LightTexelPoint;
use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;
use crate::light::lightmap::lit::lightmap_resampler::LightmapResampler;
use crate::light::lightmap::lit::lit_chart::LitChart;
use crate::light::lightmap::lit::lit_chart_next::LitChartNext;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_occluders::LightOccluders;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_sources::LightSources;
use crate::ray::ray_scene::RayScene;

/// What lights a level's charts, `CDeflector::Light` (`xrLC_Light/xrDeflectorLight.cpp`): the scene, its lights of
/// each role, the ray model they are traced through and what each of its faces lets through, and each mesh face's
/// index in it, which a texel's own face is skipped by.
pub(crate) struct LightChartLighter<'a> {
  pub(crate) scene: &'a LightChartScene<'a>,
  pub(crate) lights: &'a LightSources,
  /// The ray model the lights are traced through, on either backend.
  pub(crate) rays: RayScene<'a>,
  /// Each mesh face's index in the ray model ([`crate::GeometryRayModel::get_static_faces`]).
  pub(crate) ray_faces: &'a [Option<u32>],
  /// The steps of 255 a chart may stray from its mean and be one colour, `(4 + rms_zero) / 2`.
  pub(crate) rms_zero: u32,
  /// The steps of 255 a chart may lose shrunk and grown again and be shrunk, `(8 + rms) / 2`.
  pub(crate) rms_shrink: u32,
}

impl LightChartLighter<'_> {
  /// The light and border `chart` takes, as `CDeflector::Light` leaves it. The chart is lit at its size
  /// ([`Self::calculate`]) and its unlit texels filled ring by ring. Where every lit texel and the first ring lie
  /// within `rms_zero` of their mean (the hemisphere a third more), it is that one colour, two texels a side. Else each
  /// side tries half, then a quarter or three quarters, kept where the chart shrunk to it by Lanczos and grown back by
  /// the cubic stays within `rms_shrink` on every texel lit or in the first two rings, and the chart is lit again at
  /// the smaller size. Then it takes its border: a side one texel across a strip two texels wide of its texels, any
  /// other its lit texels and the first ring set in a texel's frame, the rest filled ring by ring.
  pub(crate) fn light(&self, chart: &LightChart) -> LitChart {
    match self.settle(self.calculate(chart, chart.width, chart.height)) {
      LitChartNext::Lit(lit) => lit,
      LitChartNext::Relit(width, height) => Self::expand(self.calculate(chart, width, height)),
      LitChartNext::Bordered(layer) => Self::expand(layer),
    }
  }

  /// What a chart's first `layer` leads to: filled ring by ring, one colour where it may be, else lit again at a
  /// smaller size where it may shrink, else bordered as it is.
  pub(crate) fn settle(&self, mut layer: LightmapLayer) -> LitChartNext {
    layer.fill_borders(254);

    if let Some(color) = self.get_one_colour(&layer) {
      return LitChartNext::Lit(LitChart {
        width: 0,
        height: 0,
        layer: LightmapLayer::filled(2, 2, color),
      });
    }

    match self.get_shrunk(&layer) {
      Some((width, height)) => LitChartNext::Relit(width, height),
      None => LitChartNext::Bordered(layer),
    }
  }

  /// `CDeflector::L_Calculate`: `chart` lit at `width` by `height` texels ([`Self::get_points`]), its points lit on the
  /// CPU, each light first tested against the face that last stopped it on the chart.
  fn calculate(&self, chart: &LightChart, width: u32, height: u32) -> LightmapLayer {
    let points: Vec<LightTexelPoint> = self.get_points(chart, width, height);
    let mut occluders: LightOccluders = LightOccluders::new(self.lights);
    let colors: Vec<LightColor> = points
      .iter()
      .map(|point| self.light_point(point.face, point.point, point.normal, &mut occluders))
      .collect();

    Self::assemble(width, height, &points, &colors)
  }

  /// The layer `width` by `height` of `points` lit `colors`: each texel the mean of its samples' light, an edge's texel
  /// its point's, halved, stored.
  pub(crate) fn assemble(width: u32, height: u32, points: &[LightTexelPoint], colors: &[LightColor]) -> LightmapLayer {
    let mut layer: LightmapLayer = LightmapLayer::new(width, height);
    let mut sums: Vec<(LightColor, u32)> = vec![(LightColor::default(), 0); (width * height) as usize];

    for (point, color) in points.iter().zip(colors) {
      let texel: usize = point.texel as usize;

      if point.is_edge {
        layer.surface[texel] = color.get_scaled(0.5).get_stored();
        layer.marker[texel] = LightmapLayer::LIT;
        continue;
      }

      let (sum, count) = &mut sums[texel];

      *sum = LightColor {
        rgb: sum.rgb + color.rgb,
        hemi: sum.hemi + color.hemi,
        sun: sum.sun + color.sun,
      };
      *count += 1;
    }

    for (texel, (sum, count)) in sums.iter().enumerate() {
      if *count > 0 {
        layer.surface[texel] = sum.get_scaled(1.0 / *count as f32).get_scaled(0.5).get_stored();
        layer.marker[texel] = LightmapLayer::LIT;
      }
    }

    layer
  }

  /// The points `chart` is lit at `width` by `height` texels, texel by texel, then the edges' (`L_Calculate`). A
  /// texel's points are its jitter samples that fall on a face, the first face of the chart's order holding the sample,
  /// each at its place on the face facing its corners' normals blended there and joined with the face's own
  /// (`L_Direct`); then each face's edges are walked in steps of an eighth of a texel, every texel they reach that has
  /// no sample and no walk yet taking the point of the edge there, facing the face (`L_Direct_Edge`). Which texel takes
  /// which is geometry alone, known before any light.
  pub(crate) fn get_points(&self, chart: &LightChart, width: u32, height: u32) -> Vec<LightTexelPoint> {
    let mut is_taken: Vec<bool> = vec![false; (width * height) as usize];
    let mut points: Vec<LightTexelPoint> = Vec::new();
    let triangles: Vec<[Vector2d; 3]> =
      chart.remap([0, 0], [width, height], [width, height], LightmapOrientation::Upright);
    let jitter: &[[f32; 2]] = self.scene.get_jitter();
    let (dimension_x, dimension_y) = (width as f32, height as f32);
    let (half_x, half_y) = (0.5 / dimension_x, 0.5 / dimension_y);
    let (spread_x, spread_y) = (0.4999 / dimension_x, 0.4999 / dimension_y);
    let sample = |x: u32, y: u32, [jitter_x, jitter_y]: [f32; 2]| {
      Vector2d::new(
        x as f32 / dimension_x + half_x + jitter_x * spread_x,
        y as f32 / dimension_y + half_y + jitter_y * spread_y,
      )
    };
    // Each texel's samples: the chart's face holding each, and where on it.
    let mut held: Vec<Option<(usize, [f32; 3])>> = vec![None; (width * height) as usize * jitter.len()];

    for (index, triangle) in triangles.iter().enumerate() {
      let (xs, ys) = Self::get_texels(triangle, width, height);

      for y in ys {
        for x in xs.clone() {
          for (number, offset) in jitter.iter().enumerate() {
            let slot: usize = (y * width + x) as usize * jitter.len() + number;

            if held[slot].is_none() {
              held[slot] = Self::get_barycentric(triangle, &sample(x, y, *offset)).map(|weights| (index, weights));
            }
          }
        }
      }
    }

    for texel in 0..(width * height) as usize {
      for (index, weights) in held[texel * jitter.len()..(texel + 1) * jitter.len()].iter().flatten() {
        let face: u32 = chart.faces[*index];
        let (point, normal) = self.get_sample(face, weights);

        is_taken[texel] = true;
        points.push(LightTexelPoint {
          texel: texel as u32,
          face,
          point,
          normal,
          is_edge: false,
        });
      }
    }

    let texel_size: f32 = (1.0 / width.max(height) as f32) / 8.0;

    for (face, triangle) in chart.faces.iter().zip(&triangles) {
      let corners: [Vector3d; 3] = self.scene.mesh.faces[*face as usize].get_corners(&self.scene.mesh.vertices);

      for (first, second) in [(0, 1), (1, 2), (2, 0)] {
        let (start, end) = (triangle[first], triangle[second]);
        let (size_x, size_y) = (end.x - start.x, end.y - start.y);
        let steps: i64 = ((size_x.abs() / texel_size).ceil() as i64).max((size_y.abs() / texel_size).ceil() as i64);

        if steps <= 0 {
          continue;
        }

        let (from, to) = (corners[first], corners[second]);
        let along: Vector3d = Vector3d::new(to.x - from.x, to.y - from.y, to.z - from.z);

        for step in 0..=steps {
          let time: f32 = step as f32 / steps as f32;
          let x: i64 = ((size_x * time + start.x) * dimension_x).floor() as i64;
          let y: i64 = ((size_y * time + start.y) * dimension_y).floor() as i64;

          if !(0..i64::from(width)).contains(&x) || !(0..i64::from(height)).contains(&y) {
            continue;
          }

          let texel: usize = (y as u32 * width + x as u32) as usize;

          if is_taken[texel] {
            continue;
          }

          let normal: &Vector3d = &self.scene.face_normals[*face as usize];

          is_taken[texel] = true;
          points.push(LightTexelPoint {
            texel: texel as u32,
            face: *face,
            point: Vec3::new(
              from.x + along.x * time,
              from.y + along.y * time,
              from.z + along.z * time,
            ),
            normal: Vec3::new(normal.x, normal.y, normal.z),
            is_edge: true,
          });
        }
      }
    }

    points
  }

  /// The point at `weights` on `face`: its corners blended there, facing its corners' normals blended there and joined
  /// with its own normal (`exact_normalize` each time).
  fn get_sample(&self, face: u32, weights: &[f32; 3]) -> (Vec3, Vec3) {
    let normals: &GeometryNormals = self.scene.normals;
    let splits: [u32; 3] = normals.faces[face as usize];
    let point: Vector3d = Self::blend(splits.map(|split| normals.positions[split as usize]), weights);
    let blended: Vector3d = GeometryNormals::exact_normalize(&Self::blend(
      splits.map(|split| normals.normals[split as usize]),
      weights,
    ));
    let own: &Vector3d = &self.scene.face_normals[face as usize];
    let normal: Vector3d =
      GeometryNormals::exact_normalize(&Vector3d::new(blended.x + own.x, blended.y + own.y, blended.z + own.z));

    (
      Vec3::new(point.x, point.y, point.z),
      Vec3::new(normal.x, normal.y, normal.z),
    )
  }

  /// `values` at a face's corners blended by `weights` on them.
  pub(crate) fn blend(values: [Vector3d; 3], weights: &[f32; 3]) -> Vector3d {
    Vector3d::new(
      values[0].x * weights[0] + values[1].x * weights[1] + values[2].x * weights[2],
      values[0].y * weights[0] + values[1].y * weights[1] + values[2].y * weights[2],
      values[0].z * weights[0] + values[1].z * weights[1] + values[2].z * weights[2],
    )
  }

  /// `LightPoint` at `point` facing `normal` from every light, `face`'s own triangle letting everything through, each
  /// light first tested against the face `occluders` remembers last stopping it whole.
  pub(crate) fn light_point(&self, face: u32, point: Vec3, normal: Vec3, occluders: &mut LightOccluders) -> LightColor {
    let skipped: Option<u32> = self.ray_faces.get(face as usize).copied().flatten();
    let light_point: LightPoint = LightPoint {
      variant: LightPointVariant::Level,
    };

    light_point.evaluate_remembering(
      &LightSamplePoint::new(point, normal),
      self.lights,
      &LevelLightRole::ALL,
      self.rays.tracer,
      |hit: &TraceHit| {
        if Some(hit.face) == skipped {
          0.0
        } else {
          self.rays.shadows.get_opacity(hit)
        }
      },
      occluders,
    )
  }

  /// `compress_Zero`: the lit texels' and first ring's mean, as stored, where every one of them lies within `rms_zero`
  /// steps of it, the hemisphere a third more; none where one strays or none is lit.
  fn get_one_colour(&self, layer: &LightmapLayer) -> Option<LightColor> {
    let near: Vec<&LightColor> = layer
      .surface
      .iter()
      .zip(&layer.marker)
      .filter(|(_, marker)| **marker >= 254)
      .map(|(color, _)| color)
      .collect();

    if near.is_empty() {
      return None;
    }

    let sum: LightColor = near.iter().fold(LightColor::default(), |sum, color| LightColor {
      rgb: sum.rgb + color.rgb,
      hemi: sum.hemi + color.hemi,
      sun: sum.sun + color.sun,
    });
    let mean: LightColor = sum.get_scaled(1.0 / near.len() as f32);
    let bytes =
      |color: &LightColor| [color.rgb.x, color.rgb.y, color.rgb.z, color.sun, color.hemi].map(LightColor::get_byte);
    let target: [u8; 5] = bytes(&mean);
    let limit: u32 = self.rms_zero;

    near
      .iter()
      .all(|color| {
        bytes(color)
          .iter()
          .zip(&target)
          .enumerate()
          .all(|(term, (byte, target))| {
            let apart: u32 = u32::from(byte.abs_diff(*target));

            if term == 4 {
              apart <= limit * 4 / 3
            } else {
              apart <= limit
            }
          })
      })
      .then(|| mean.get_stored())
  }

  /// `compress_RMS`: the smaller size the chart may be lit at, none where neither side may shrink. Each side of two
  /// texels or more tries half; where that holds a quarter, else three quarters; the other side as it is.
  fn get_shrunk(&self, layer: &LightmapLayer) -> Option<(u32, u32)> {
    let (width, height) = (layer.width, layer.height);
    let side = |length: u32, test: &dyn Fn(u32) -> bool| {
      if length < 2 {
        return None;
      }

      if test(length / 2) {
        Some(if test(length / 4) { length / 4 } else { length / 2 })
      } else {
        let three_quarters: u32 = length * 3 / 4;

        test(three_quarters).then_some(three_quarters)
      }
    };
    let packed: Vec<[u8; 4]> = layer.get_packed();
    let shrunk_width: Option<u32> = side(width, &|width| self.holds(layer, &packed, width, height));
    let shrunk_height: Option<u32> = side(height, &|height| self.holds(layer, &packed, width, height));

    (shrunk_width.is_some() || shrunk_height.is_some())
      .then(|| (shrunk_width.unwrap_or(width), shrunk_height.unwrap_or(height)))
  }

  /// `rms_test`: whether `layer`, its texels `packed`, shrunk to `width` by `height` and grown back stays within
  /// `rms_shrink` steps on each of its lit and first two rings' texels. The source's hemisphere test reads the sun
  /// again, held already, so the hemisphere is not tested.
  fn holds(&self, layer: &LightmapLayer, packed: &[[u8; 4]], width: u32, height: u32) -> bool {
    if width <= 1 || height <= 1 {
      return false;
    }

    let shrunk: Vec<[u8; 4]> = LightmapResampler::Lanczos3.resample(packed, layer.width, layer.height, width, height);
    let restored: Vec<[u8; 4]> = LightmapResampler::Cubic.resample(&shrunk, width, height, layer.width, layer.height);

    layer
      .marker
      .iter()
      .zip(packed.iter().zip(&restored))
      .filter(|(marker, _)| **marker >= LightmapLayer::TAKEN)
      .all(|(_, (original, restored))| {
        original
          .iter()
          .zip(restored)
          .all(|(original, restored)| u32::from(original.abs_diff(*restored)) <= self.rms_shrink)
      })
  }

  /// The border round a lit `layer`: a side one texel across, a strip two texels wide of its texels, every one lit;
  /// any other, its lit and first-ring texels in a frame a texel wide, filled ring by ring.
  pub(crate) fn expand(layer: LightmapLayer) -> LitChart {
    let (width, height) = (layer.width, layer.height);
    let border: u32 = LightChart::BORDER;

    if width == 1 {
      let mut strip: LightmapLayer = LightmapLayer::new(2 * border, height + 2 * border);

      for y in 0..strip.height {
        let color: LightColor = layer.surface[(i64::from(y) - 1).clamp(0, i64::from(height) - 1) as usize];

        for x in 0..2 {
          let texel: usize = strip.get_index(x, y);

          strip.surface[texel] = color;
          strip.marker[texel] = LightmapLayer::LIT;
        }
      }

      return LitChart {
        width: 0,
        height,
        layer: strip,
      };
    }

    if height == 1 {
      let mut strip: LightmapLayer = LightmapLayer::new(width + 2 * border, 2 * border);

      for x in 0..strip.width {
        let color: LightColor = layer.surface[(i64::from(x) - 1).clamp(0, i64::from(width) - 1) as usize];

        for y in 0..2 {
          let texel: usize = strip.get_index(x, y);

          strip.surface[texel] = color;
          strip.marker[texel] = LightmapLayer::LIT;
        }
      }

      return LitChart {
        width,
        height: 0,
        layer: strip,
      };
    }

    let mut framed: LightmapLayer = LightmapLayer::new(width + 2 * border, height + 2 * border);

    for y in 0..height {
      for x in 0..width {
        let from: usize = layer.get_index(x, y);

        if layer.marker[from] >= 254 {
          let to: usize = framed.get_index(x + border, y + border);

          framed.surface[to] = layer.surface[from];
          framed.marker[to] = layer.marker[from];
        }
      }
    }

    for reference in [254, 253, 252, 251] {
      framed.apply_borders(reference);
    }

    framed.fill_borders(250);

    LitChart {
      width,
      height,
      layer: framed,
    }
  }

  /// The texels whose samples may fall on `triangle`, over a layer `width` by `height`.
  fn get_texels(triangle: &[Vector2d; 3], width: u32, height: u32) -> (std::ops::Range<u32>, std::ops::Range<u32>) {
    let range = |values: [f32; 3], side: u32| {
      let low: f32 = values.into_iter().fold(f32::INFINITY, f32::min);
      let high: f32 = values.into_iter().fold(f32::NEG_INFINITY, f32::max);
      let first: i64 = ((low * side as f32).floor() as i64 - 1).max(0);
      let last: i64 = ((high * side as f32).ceil() as i64 + 1).min(i64::from(side) - 1);

      first as u32..(last + 1).max(first) as u32
    };

    (
      range(triangle.map(|corner| corner.x), width),
      range(triangle.map(|corner| corner.y), height),
    )
  }

  /// `_TCF::barycentric`: `point`'s weights on `triangle`'s corners, where each lies between 0 and 1 (`isInside`).
  pub(crate) fn get_barycentric(triangle: &[Vector2d; 3], point: &Vector2d) -> Option<[f32; 3]> {
    let [first, second, third] = triangle;
    let (v02_x, v02_y) = (first.x - third.x, first.y - third.y);
    let (v12_x, v12_y) = (second.x - third.x, second.y - third.y);
    let (pv2_x, pv2_y) = (point.x - third.x, point.y - third.y);
    let m00: f32 = v02_x * v02_x + v02_y * v02_y;
    let m01: f32 = v02_x * v12_x + v02_y * v12_y;
    let m11: f32 = v12_x * v12_x + v12_y * v12_y;
    let r0: f32 = v02_x * pv2_x + v02_y * pv2_y;
    let r1: f32 = v12_x * pv2_x + v12_y * pv2_y;
    let determinant: f32 = m00 * m11 - m01 * m01;
    let u: f32 = (m11 * r0 - m01 * r1) / determinant;
    let v: f32 = (m00 * r1 - m01 * r0) / determinant;
    let w: f32 = 1.0 - u - v;

    [u, v, w]
      .iter()
      .all(|weight| (0.0..=1.0).contains(weight))
      .then_some([u, v, w])
  }
}
