use std::collections::HashMap;

use xrf_math::{EPS_S, Vector2d, Vector3d};

use crate::chart::light_chart_scene::LightChartScene;
use crate::chart::lightmap_placement::LightmapPlacement;

/// One lightmap chart, xrLC's deflector (`CDeflector`, `xrLC_Light/xrDeflector.cpp`): faces of one material that lie
/// within the smoothing angle of one face, projected along their mean normal onto a plane, and the texels a side that
/// plane takes at the bake's density.
#[derive(Clone, Debug, PartialEq)]
pub struct LightChart {
  /// Its faces, by index into the mesh's faces, in the order they were placed on it (`UVpolys`).
  pub faces: Vec<u32>,
  /// The normal it is projected along.
  pub normal: Vector3d,
  /// Each face's corners projected onto its plane, in metres.
  pub projected: Vec<[Vector2d; 3]>,
  /// Its texels a side before borders.
  pub width: u32,
  pub height: u32,
  /// Its faces' material (`GetBaseMaterial`).
  pub material: u16,
}

impl LightChart {
  /// `BORDER`: the texels a chart's mask reaches past its own on each side.
  pub const BORDER: u32 = 1;

  /// `c_LMAP_size`: a lightmap page's side in texels.
  pub const PAGE_SIDE: u32 = 1024;

  /// The most texels a chart takes a side, `512 - 2 * BORDER` (`xrDeflector.cpp:196`).
  pub const MAXIMUM_SIDE: u32 = 512 - 2 * Self::BORDER;

  /// `EPS`: how much a face's area weighs its normal into its chart's (`OA_Export`).
  const AREA_WEIGHT: f32 = 0.000_01;

  /// `xrPhase_UVmap`: the charts one material split of `scene` is cut into, in the order they are cut. Each starts from
  /// the largest face not yet charted, the first of equals, and takes, depth first through the faces sharing a split
  /// vertex in the split's order (`Face::OA_Unwarp`), every face within the smoothing angle and a degree of that face's
  /// normal (`CDeflector::OA_Place`); each is then sized ([`Self::of`]). A face of no area starts none.
  pub fn cut(split: &[u32], scene: &LightChartScene<'_>) -> Vec<Self> {
    let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();

    for face in split {
      for vertex in scene.normals.faces[*face as usize] {
        adjacency.entry(vertex).or_default().push(*face);
      }
    }

    let limit: f32 = (scene.smooth_angle + 1.0).to_radians().cos();
    let mut is_charted: HashMap<u32, bool> = split.iter().map(|face| (*face, false)).collect();
    let mut charts: Vec<Self> = Vec::new();
    // The seeds in the order they are taken: the largest first, the first of equals, none of no area.
    let mut seeds: Vec<u32> = split
      .iter()
      .copied()
      .filter(|face| scene.areas[*face as usize] > 0.0)
      .collect();

    seeds.sort_by(|first, second| scene.areas[*second as usize].total_cmp(&scene.areas[*first as usize]));

    for seed in seeds {
      if is_charted[&seed] {
        continue;
      }

      let normal: Vector3d = Self::normalize(&scene.face_normals[seed as usize]);
      let accepts = |face: u32| normal.dot(&scene.face_normals[face as usize]) >= limit;
      let mut faces: Vec<u32> = Vec::new();
      // Each frame: a face placed on the chart, the corner and the adjacent face it goes on from.
      let mut stack: Vec<(u32, usize, usize)> = Vec::new();
      let mut visit = |face: u32, faces: &mut Vec<u32>, stack: &mut Vec<(u32, usize, usize)>| {
        let charted: &mut bool = is_charted.get_mut(&face).expect("a face of the split");

        if !*charted && accepts(face) {
          *charted = true;
          faces.push(face);
          stack.push((face, 0, 0));
        }
      };

      visit(seed, &mut faces, &mut stack);

      while let Some((face, corner, next)) = stack.last_mut() {
        if *corner == 3 {
          stack.pop();
          continue;
        }

        let vertex: u32 = scene.normals.faces[*face as usize][*corner];
        let adjacent: &[u32] = &adjacency[&vertex];

        if *next == adjacent.len() {
          *corner += 1;
          *next = 0;
          continue;
        }

        let neighbour: u32 = adjacent[*next];

        *next += 1;
        visit(neighbour, &mut faces, &mut stack);
      }

      charts.push(Self::of(faces, scene));
    }

    charts
  }

  /// `CDeflector::OA_Export`: the chart of `faces`, in their order. Its normal is the faces' normals summed, each
  /// weighed by a hair of its area, and normalised; its plane is the view `build_camera` looks along it from, up being
  /// the world's up or, for a normal near it, the world's x; and each side is the projected rectangle's at the density
  /// times the faces' mean light density, rounded up from a half texel on, between 1 and [`Self::MAXIMUM_SIDE`].
  pub fn of(faces: Vec<u32>, scene: &LightChartScene<'_>) -> Self {
    let mut sum: Vector3d = Vector3d::ZERO;
    let mut density: f32 = 0.0;

    for face in &faces {
      let normal: &Vector3d = &scene.face_normals[*face as usize];
      let weight: f32 = 1.0 + Self::AREA_WEIGHT * scene.areas[*face as usize];
      let material: u16 = scene.mesh.faces[*face as usize].material;

      sum = Vector3d::new(
        sum.x + normal.x * weight,
        sum.y + normal.y * weight,
        sum.z + normal.z * weight,
      );
      density += scene.densities.get(usize::from(material)).copied().unwrap_or(0.0);
    }

    let length: f32 = (sum.x * sum.x + sum.y * sum.y + sum.z * sum.z).sqrt();
    let normal: Vector3d = if length > EPS_S && length.is_finite() {
      Self::normalize(&sum)
    } else {
      Vector3d::new(0.0, 1.0, 0.0)
    };

    density /= faces.len() as f32;

    let projected: Vec<[Vector2d; 3]> = Self::project(&faces, &normal, scene);
    let (minimum, maximum) = Self::get_rect(&projected);
    let side =
      |extent: f32| ((extent * scene.pixels_per_meter * density + 0.5).ceil() as u32).clamp(1, Self::MAXIMUM_SIDE);

    Self {
      material: faces
        .first()
        .map_or(0, |face| scene.mesh.faces[*face as usize].material),
      width: side(maximum.x - minimum.x),
      height: side(maximum.y - minimum.y),
      faces,
      normal,
      projected,
    }
  }

  /// `CDeflector::RemapUV`: each face's corners mapped from the projected rectangle into the texels `size` from `base`
  /// of a texture `texture` a side, a half texel in from each edge, crosswise where `is_rotated`; an empty span is a
  /// hair either side of its middle.
  pub fn remap(&self, base: [u32; 2], size: [u32; 2], texture: [u32; 2], is_rotated: bool) -> Vec<[Vector2d; 3]> {
    let (minimum, maximum) = Self::get_rect(&self.projected);
    let extent: Vector2d = Vector2d::new(maximum.x - minimum.x, maximum.y - minimum.y);
    let span = |axis: usize| {
      let low: f32 = (base[axis] as f32 + 0.5) / texture[axis] as f32;
      let high: f32 = ((base[axis] + size[axis]) as f32 - 0.5) / texture[axis] as f32;

      if low >= high {
        let middle: f32 = low.midpoint(high);

        (middle - EPS_S, middle + EPS_S)
      } else {
        (low, high)
      }
    };
    let ((low_u, high_u), (low_v, high_v)) = (span(0), span(1));
    let (size_u, size_v) = (high_u - low_u, high_v - low_v);

    self
      .projected
      .iter()
      .map(|corners| {
        corners.map(|corner| {
          if is_rotated {
            Vector2d::new(
              ((corner.y - minimum.y) / extent.y) * size_u + low_u,
              ((corner.x - minimum.x) / extent.x) * size_v + low_v,
            )
          } else {
            Vector2d::new(
              ((corner.x - minimum.x) / extent.x) * size_u + low_u,
              ((corner.y - minimum.y) / extent.y) * size_v + low_v,
            )
          }
        })
      })
      .collect()
  }

  /// `CLightmap::Capture`: each face's lightmap coordinates where the chart was placed, its rectangle less the border
  /// on each side.
  pub fn get_page_coordinates(&self, placement: &LightmapPlacement) -> Vec<[Vector2d; 3]> {
    let border: u32 = Self::BORDER;

    self.remap(
      [placement.x + border, placement.y + border],
      [placement.width - 2 * border, placement.height - 2 * border],
      [Self::PAGE_SIDE, Self::PAGE_SIDE],
      placement.is_rotated,
    )
  }

  /// Each face's corners in the view `build_camera(normal, 0, up)` gives (`transform_tiny`).
  fn project(faces: &[u32], normal: &Vector3d, scene: &LightChartScene<'_>) -> Vec<[Vector2d; 3]> {
    let world_up: Vector3d = if normal.y.abs() > 0.99 {
      Vector3d::new(1.0, 0.0, 0.0)
    } else {
      Vector3d::new(0.0, 1.0, 0.0)
    };
    let right: Vector3d = Self::normalize_safe(&Self::cross(&world_up, normal));
    let up: Vector3d = Self::normalize_safe(&Self::cross(normal, &right));
    // `build_camera`, from the normal towards the origin.
    let view: Vector3d = Self::normalize(&Vector3d::new(-normal.x, -normal.y, -normal.z));
    let along: f32 = up.dot(&view);
    let view_up: Vector3d = Self::normalize(&Vector3d::new(
      view.x * -along + up.x,
      view.y * -along + up.y,
      view.z * -along + up.z,
    ));
    let view_right: Vector3d = Self::cross(&view_up, &view);
    let (shift_x, shift_y) = (-normal.dot(&view_right), -normal.dot(&view_up));

    faces
      .iter()
      .map(|face| {
        scene.mesh.faces[*face as usize]
          .get_corners(&scene.mesh.vertices)
          .map(|corner| {
            Vector2d::new(
              corner.x * view_right.x + corner.y * view_right.y + corner.z * view_right.z + shift_x,
              corner.x * view_up.x + corner.y * view_up.y + corner.z * view_up.z + shift_y,
            )
          })
      })
      .collect()
  }

  /// `CDeflector::GetRect`: the projected corners' least and greatest coordinates.
  fn get_rect(projected: &[[Vector2d; 3]]) -> (Vector2d, Vector2d) {
    let first: Vector2d = projected.first().map_or(Vector2d::new(0.0, 0.0), |corners| corners[0]);

    projected
      .iter()
      .flatten()
      .fold((first, first), |(minimum, maximum), corner| {
        (
          Vector2d::new(minimum.x.min(corner.x), minimum.y.min(corner.y)),
          Vector2d::new(maximum.x.max(corner.x), maximum.y.max(corner.y)),
        )
      })
  }

  /// `Fvector::crossproduct`.
  fn cross(first: &Vector3d, second: &Vector3d) -> Vector3d {
    Vector3d::new(
      first.y * second.z - first.z * second.y,
      first.z * second.x - first.x * second.z,
      first.x * second.y - first.y * second.x,
    )
  }

  /// `Fvector::normalize`: by the square root of the inverse square length.
  fn normalize(vector: &Vector3d) -> Vector3d {
    let scale: f32 = (1.0 / (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z)).sqrt();

    Vector3d::new(vector.x * scale, vector.y * scale, vector.z * scale)
  }

  /// `Fvector::normalize_safe`: normalised where its square length is a normal float, else as it is.
  fn normalize_safe(vector: &Vector3d) -> Vector3d {
    if vector.x * vector.x + vector.y * vector.y + vector.z * vector.z > f32::MIN_POSITIVE {
      Self::normalize(vector)
    } else {
      *vector
    }
  }
}
