use glam::Vec3;
use rayon::prelude::*;
use xrf_geometry::BvhTracer;
use xrf_level::LevelLightRole;
use xrf_math::Vector3d;

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::shadow::shadow_materials::ShadowMaterials;

/// Every vertex's sky term as xrLC computes it before the smoothing split (`xrPhase_AdaptiveHT`,
/// `xrLC/xrPhase_AdaptiveHT.cpp`), which every vertex split from it keeps: each welded vertex of the cut mesh, its
/// normal the sum of its faces' (`normalFromAdj`), lit by the sky alone through the ray model of that mesh
/// ([`crate::GeometryRayModel::build_welded`]) and halved; then `Self::SMOOTHING_ROUNDS` rounds in which each vertex
/// takes the mean of every vertex of its faces, itself included (`u_SmoothVertColors`). Each term is kept as
/// `base_color` stores it.
pub struct LightHemisphere;

impl LightHemisphere {
  /// `u_SmoothVertColors(5)`.
  pub(crate) const SMOOTHING_ROUNDS: usize = 5;

  /// The sky term of each of `mesh`'s vertices, by index, the sky's lights `sky`.
  pub fn of(mesh: &GeometryMesh, sky: &[LightSource], tracer: &BvhTracer, shadows: &ShadowMaterials) -> Vec<f32> {
    let face_normals: Vec<Vector3d> = mesh
      .faces
      .iter()
      .map(|face| GeometryNormals::get_face_normal(&face.get_corners(&mesh.vertices)))
      .collect();
    let mut adjacent: Vec<Vec<u32>> = vec![Vec::new(); mesh.vertices.len()];

    for (index, face) in mesh.faces.iter().enumerate() {
      for vertex in face.vertices {
        adjacent[vertex as usize].push(index as u32);
      }
    }

    let light_point: LightPoint = LightPoint {
      variant: LightPointVariant::Level,
    };
    let lights: LightSources = LightSources::only_sky(sky);
    let mut terms: Vec<f32> = (0..mesh.vertices.len())
      .into_par_iter()
      .map(|vertex| {
        if adjacent[vertex].is_empty() {
          return 0.0;
        }

        let mut sum: Vector3d = Vector3d::ZERO;

        for face in &adjacent[vertex] {
          sum = &sum + &face_normals[*face as usize];
        }

        let normal: Vector3d = GeometryNormals::exact_normalize(&sum);
        let position: Vector3d = mesh.vertices[vertex];
        let lit: LightColor = light_point.evaluate(
          &LightSamplePoint::new(
            Vec3::new(position.x, position.y, position.z),
            Vec3::new(normal.x, normal.y, normal.z),
          ),
          &lights,
          &[LevelLightRole::Hemi],
          tracer,
          shadows,
        );

        LightColor::get_fixed(lit.hemi * 0.5)
      })
      .collect();
    let circles: Vec<Vec<u32>> = adjacent
      .iter()
      .map(|faces| {
        let mut circle: Vec<u32> = faces
          .iter()
          .flat_map(|face| mesh.faces[*face as usize].vertices)
          .collect();

        circle.sort_unstable();
        circle.dedup();
        circle
      })
      .collect();

    for _ in 0..Self::SMOOTHING_ROUNDS {
      terms = circles
        .iter()
        .map(|circle| {
          let sum: f32 = circle.iter().map(|vertex| terms[*vertex as usize]).sum();

          LightColor::get_fixed(sum * (1.0 / circle.len() as f32))
        })
        .collect();
    }

    terms
  }
}
