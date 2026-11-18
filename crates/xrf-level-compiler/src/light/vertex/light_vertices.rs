use std::collections::HashMap;

use glam::Vec3;
use rayon::prelude::*;
use xrf_geometry::{PointWelder, XrlcTracer};
use xrf_level::LevelLightRole;

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::input::compiler_shaders::CompilerShaders;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_occluders::LightOccluders;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_sources::LightSources;
use crate::shadow::shadow_materials::ShadowMaterials;

/// Every split vertex's light as xrLC leaves it in the vertex (`LightVertex`, `xrLC_Light/xrLightVertex.cpp`): the sky
/// term of the welded vertex it was split from ([`LightHemisphere`](super::light_hemisphere::LightHemisphere)), and for
/// a vertex of a vertex-lit face also
/// the static light and the sun from every light, halved; then, among the vertex-lit vertices within `EPS_L` of the
/// first at a place, each moved toward the brightest of them by its faces' mean `vert_translucency`, never darker. Each
/// term is kept as `base_color` stores it.
///
/// The rays go through xrLC's own ray model ([`XrlcTracer`]), as each vertex is matched to xrLC's: a shadow ray
/// grazing an axis-aligned wall exactly at its edge passes in xrLC where a tighter tree stops it. The lightmaps keep
/// [`xrf_geometry::BvhTracer`], which the GPU backend mirrors byte for byte; a texel's light is matched in distribution,
/// not texel for texel, so the rays xrLC loses there do not show.
///
/// xrLC's threads take the vertices in order one at a time and share each light's remembered blocker (`rayTrace`'s
/// `L.tri`), which a vertex's ray is tested against before the query: where the tree loses a ray at a corner, the
/// blocker of the vertex lit just before can still stop it. Each run of `Self::REMEMBERED_RUN` vertices is lit in
/// order the same way, remembering from one vertex to the next.
pub(crate) struct LightVertices;

impl LightVertices {
  /// Vertices lit in order by one lighter, which remembers each light's last blocker from one to the next.
  const REMEMBERED_RUN: usize = 1024;

  /// The light of each of `normals`' split vertices of `mesh`, by index, the welded vertices' sky terms `hemispheres`,
  /// the level's ray model in `tracer`.
  pub(crate) fn of(
    mesh: &GeometryMesh,
    normals: &GeometryNormals,
    shaders: &CompilerShaders,
    hemispheres: &[f32],
    lights: &LightSources,
    tracer: &XrlcTracer,
    shadows: &ShadowMaterials,
  ) -> Vec<LightColor> {
    let count: usize = normals.positions.len();
    let mut welded: Vec<u32> = vec![0; count];
    let mut translucency: Vec<(f32, u32)> = vec![(0.0, 0); count];
    let mut is_vertex_lit: Vec<bool> = vec![false; count];

    for (face, corners) in mesh.faces.iter().zip(&normals.faces) {
      let shader = shaders.get(face.material);
      let value: f32 = shader.map_or(0.0, |shader| shader.vertex_translucency);
      let lit: bool = shader.is_some_and(|shader| shader.is_vertex_lit());

      for (corner, split) in corners.iter().enumerate() {
        let split: usize = *split as usize;

        welded[split] = face.vertices[corner];
        translucency[split].0 += value;
        translucency[split].1 += 1;
        is_vertex_lit[split] |= lit;
      }
    }

    let light_point: LightPoint = LightPoint {
      variant: LightPointVariant::Level,
    };
    let indices: Vec<usize> = (0..count).collect();
    let colors: Vec<LightColor> = indices
      .par_chunks(Self::REMEMBERED_RUN)
      .flat_map_iter(|run| {
        let mut occluders: LightOccluders = LightOccluders::new(lights);

        run
          .iter()
          .map(|split| {
            let split: usize = *split;
            let hemi: f32 = hemispheres[welded[split] as usize];

            if !is_vertex_lit[split] {
              return LightColor {
                hemi,
                ..LightColor::default()
              };
            }

            let position = &normals.positions[split];
            let normal = &normals.normals[split];
            let point: LightSamplePoint = LightSamplePoint::new(
              Vec3::new(position.x, position.y, position.z),
              Vec3::new(normal.x, normal.y, normal.z),
            );
            let lit: LightColor = light_point
              .evaluate_remembering(
                &point,
                lights,
                &[LevelLightRole::Rgb, LevelLightRole::Sun],
                tracer,
                |hit| shadows.get_opacity(hit),
                &mut occluders,
              )
              .get_scaled(0.5)
              .get_stored();

            LightColor { hemi, ..lit }
          })
          .collect::<Vec<LightColor>>()
      })
      .collect();

    Self::get_translucent(normals, &colors, &is_vertex_lit, &translucency)
  }

  /// `LightVertex`'s translucency pass over the vertex-lit vertices, grouped in their order.
  fn get_translucent(
    normals: &GeometryNormals,
    colors: &[LightColor],
    is_vertex_lit: &[bool],
    translucency: &[(f32, u32)],
  ) -> Vec<LightColor> {
    let mut welder: PointWelder = PointWelder::default();
    let mut groups: Vec<Option<u32>> = vec![None; colors.len()];
    let mut brightest: HashMap<u32, LightColor> = HashMap::new();

    for split in (0..colors.len()).filter(|split| is_vertex_lit[*split]) {
      let group: u32 = welder.weld(&normals.positions[split]);
      let entry: &mut LightColor = brightest.entry(group).or_default();
      let color: &LightColor = &colors[split];

      groups[split] = Some(group);
      *entry = LightColor {
        rgb: entry.rgb.max(color.rgb),
        hemi: entry.hemi.max(color.hemi),
        sun: entry.sun.max(color.sun),
      };
    }

    colors
      .iter()
      .zip(&groups)
      .zip(translucency)
      .map(|((color, group), (sum, count))| {
        let Some(group) = group else {
          return *color;
        };
        let level: f32 = LightColor::get_fixed(sum / (*count).max(1) as f32);
        let other: &LightColor = &brightest[group];
        let unlevel: f32 = 1.0 - level;

        LightColor {
          rgb: color.rgb.lerp(other.rgb, level).max(color.rgb),
          hemi: (unlevel * color.hemi + level * other.hemi).max(color.hemi),
          sun: (unlevel * color.sun + level * other.sun).max(color.sun),
        }
        .get_stored()
      })
      .collect()
  }
}
