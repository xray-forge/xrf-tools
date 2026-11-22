use xrf_geometry::XrlcTracer;
use xrf_job::parallel::prelude::*;
use xrf_level::LevelLightRole;
use xrf_math::Vector3d;
use xrf_math::glam::Vec3;

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::shadow::shadow_materials::ShadowMaterials;

/// Every vertex's sky term as xrLC computes it before the smoothing split (`xrPhase_AdaptiveHT`), kept by every vertex
/// split from it, as `base_color` stores it.
///
/// Each welded vertex of the cut mesh, its normal its faces' summed (`normalFromAdj`), is lit by the sky alone through
/// that mesh's ray model ([`GeometryRayModel::build_welded`]) and halved; then in `Self::SMOOTHING_ROUNDS` rounds each
/// vertex takes the mean of every vertex of its faces, itself included (`u_SmoothVertColors`).
///
/// The rays go through xrLC's own ray model ([`XrlcTracer`]) to match it vertex for vertex: a sky ray grazing an
/// axis-aligned wall exactly at its edge passes there where a tighter tree stops it. Each ray is a full query: xrLC's
/// threads share each light's remembered blocker (`rayTrace`'s `L.tri`) across far-apart vertices, which changes no
/// vertex's light with the threads in step and otherwise depends on their timing.
///
/// [`GeometryRayModel::build_welded`]: crate::geometry::collision::geometry_ray_model::GeometryRayModel::build_welded
pub struct LightHemisphere;

impl LightHemisphere {
  /// `u_SmoothVertColors(5)`.
  pub const SMOOTHING_ROUNDS: usize = 5;

  /// The sky term of each of `mesh`'s vertices, by index, the sky's lights `sky`, the welded mesh's ray model in
  /// `tracer`.
  pub fn of(mesh: &GeometryMesh, sky: &[LightSource], tracer: &XrlcTracer, shadows: &ShadowMaterials) -> Vec<f32> {
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

        let normal: Vector3d = sum.exact_normalize();
        let position: Vector3d = mesh.vertices[vertex];
        let lit: LightColor = light_point.evaluate(
          &LightSamplePoint::new(Vec3::from(position), Vec3::from(normal)),
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
