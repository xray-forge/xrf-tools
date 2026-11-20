use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

use glam::Vec3;
use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_geometry::{PointWelder, TraceHit, Triangle, XrlcTracer};
use xrf_level::{BuildCformRaycastFace, BuildPrjFile, BuildPrjMuModel, LevelLightRole};
use xrf_math::{Matrix4x4, Vector3d};

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::model::geometry_placement::GeometryPlacement;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::input::compiler_shaders::CompilerShaders;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_sources::LightSources;
use crate::light::vertex::light_model_faces::LightModelFaces;
use crate::light::vertex::light_models_input::LightModelsInput;
use crate::light::vertex::light_reference::LightReference;
use crate::light::vertex::light_tree_color::LightTreeColor;

/// One multi-use model's vertices, as its own normals pass splits them, with what lighting them needs.
struct LitModel {
  positions: Vec<Vector3d>,
  normals: Vec<Vector3d>,
  /// Each vertex's faces' mean `vert_ambient` and `vert_translucency`.
  ambient: Vec<f32>,
  translucency: Vec<f32>,
  /// Each vertex's group of vertices within `EPS_L` of the first at its place.
  groups: Vec<u32>,
}

/// One lighting of a model's vertices ([`LightModels::light`]): placed by `transform`, lit by `roles` of `lights` from
/// `samples` jittered samples each, seeded by `seed`, through `tracer`.
struct LightPass<'a> {
  model: &'a LitModel,
  transform: &'a Matrix4x4,
  lights: &'a LightSources,
  roles: &'a [LevelLightRole],
  samples: u32,
  seed: u64,
  tracer: &'a XrlcTracer,
}

/// The multi-use models' light as xrLC computes it (`xrLC/xrMU_Model_Calc_lighting.cpp`).
///
/// - **A model**'s vertices are lit in its own space by the sky alone, through a ray model of its own shadow-casting
///   faces (`xrMU_Model::calc_lighting`): the sky term its trees store in each vertex.
/// - **A placement**'s vertices are lit with every term through the level's ray model
///   (`xrMU_Reference::calc_lighting`). Its tree's scale and bias for each term are a least-squares line of the
///   placement's term on the model's sky term (`vfComputeLinearRegression`), then `simple_optimize` on the model's
///   sun, which is 0, so only the bias moves toward the term's mean.
///
/// A vertex is lit from [`Self::SAMPLES`] samples ([`Self::DRAFT_SAMPLES`] at draft), each from a normal jittered
/// within `Self::JITTER_DEGREES` and a step along it, its faces' mean `vert_ambient` added to the sky. Vertices within
/// `EPS_L` of the first at a place then move toward their brightest by their mean `vert_translucency`, never darker,
/// and are halved. Where xrLC jitters from its global random source, this seeds by model, placement, vertex and
/// sample, so a compile is repeatable. Both ray models are xrLC's own ([`XrlcTracer`]).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LightModels {
  /// Each model's vertices' light, by model in `build.prj`'s order and vertex as its normals pass splits them.
  pub models: Vec<Vec<LightColor>>,
  /// Each placement's, in `build.prj`'s order.
  pub references: Vec<LightReference>,
}

impl LightModels {
  /// Samples a vertex above draft quality.
  pub(crate) const SAMPLES: u32 = 6;

  /// Samples a vertex at draft quality.
  pub(crate) const DRAFT_SAMPLES: u32 = 1;

  /// The cone a sample's normal is jittered within.
  pub(crate) const JITTER_DEGREES: f32 = 30.0;

  /// How far along its normal the last sample steps, the first not at all.
  const SAMPLE_REACH: f32 = 0.2;

  /// The faces each model lights itself through, in model space and their models' order, as records for
  /// `ShadowMaterials::load`, with where each model's start.
  ///
  /// # Errors
  ///
  /// Returns an error for a model face naming a vertex the model lacks or a material of no compiler shader.
  pub(crate) fn get_model_faces(project: &BuildPrjFile, shaders: &CompilerShaders) -> XrfResult<LightModelFaces> {
    let mut faces: Vec<BuildCformRaycastFace> = Vec::new();
    let mut starts: Vec<usize> = Vec::with_capacity(project.mu_models.len());

    for model in &project.mu_models {
      starts.push(faces.len());

      for (number, face) in model.faces.iter().enumerate() {
        if shaders.require(model, number, face)?.casts_shadow() {
          faces.push(BuildCformRaycastFace {
            material: face.material,
            material_padding: 0,
            game_material: face.game_material,
            texture_coordinates: face.texture_coordinates,
            reserved: 0,
            trailing_padding: 0,
          });
        }
      }
    }

    Ok(LightModelFaces { faces, starts })
  }

  /// The light of the input's project's models and placements: the models through their own faces, the placements
  /// through the level's ray model.
  ///
  /// # Errors
  ///
  /// Returns an error where a model cannot be loaded or a placement names a model the project lacks.
  pub(crate) fn of(input: &LightModelsInput<'_>) -> XrfResult<Self> {
    let LightModelsInput {
      project,
      shaders,
      lights,
      model_shadows,
      model_starts,
      tracer,
      shadows,
      samples,
    } = *input;
    let lit_models: Vec<LitModel> = project
      .mu_models
      .iter()
      .map(|model| Self::prepare(model, shaders))
      .collect::<XrfResult<_>>()?;
    let sky: LightSources = LightSources::only_sky(&lights.hemi);
    let models: Vec<Vec<LightColor>> = project
      .mu_models
      .par_iter()
      .zip(&lit_models)
      .enumerate()
      .map(|(index, (model, lit))| {
        let own: XrlcTracer = Self::get_model_tracer(model, shaders);
        let start: usize = model_starts[index];

        Self::light(
          &LightPass {
            model: lit,
            transform: &Matrix4x4::IDENTITY,
            lights: &sky,
            roles: &[LevelLightRole::Hemi],
            samples,
            seed: (index as u64) << 32,
            tracer: &own,
          },
          |hit| {
            model_shadows.get_opacity(&TraceHit {
              face: hit.face + start as u32,
              ..*hit
            })
          },
        )
      })
      .collect();
    let references: Vec<LightReference> = project
      .mu_references
      .par_iter()
      .enumerate()
      .map(|(index, reference)| {
        GeometryPlacement::get_model(project, index, reference)?;

        let colors: Vec<LightColor> = Self::light(
          &LightPass {
            model: &lit_models[reference.model as usize],
            transform: &reference.transform,
            lights,
            roles: &LevelLightRole::ALL,
            samples,
            seed: (1 << 63) | ((index as u64) << 32),
            tracer,
          },
          |hit| shadows.get_opacity(hit),
        );
        Ok(Self::fit(&models[reference.model as usize], colors))
      })
      .collect::<XrfResult<_>>()?;

    Ok(Self { models, references })
  }

  /// A model's vertices split by its own normals pass, and each one's ambient, translucency and group.
  fn prepare(model: &BuildPrjMuModel, shaders: &CompilerShaders) -> XrfResult<LitModel> {
    let mesh: GeometryMesh = GeometryMesh::of_model(model)?;
    let normals: GeometryNormals = GeometryNormals::of(&mesh);
    let count: usize = normals.positions.len();
    let mut sums: Vec<(f32, f32, u32)> = vec![(0.0, 0.0, 0); count];

    for (face, corners) in mesh.faces.iter().zip(&normals.faces) {
      let (ambient, translucency): (f32, f32) = shaders
        .get(face.material)
        .map_or((0.0, 0.0), |shader| (shader.vertex_ambient, shader.vertex_translucency));

      for corner in corners {
        let sum: &mut (f32, f32, u32) = &mut sums[*corner as usize];

        sum.0 += ambient;
        sum.1 += translucency;
        sum.2 += 1;
      }
    }

    let mut welder: PointWelder = PointWelder::default();
    let groups: Vec<u32> = normals.positions.iter().map(|position| welder.weld(position)).collect();
    let mean = |sum: f32, count: u32| sum / count.max(1) as f32;

    Ok(LitModel {
      ambient: sums.iter().map(|(sum, _, count)| mean(*sum, *count)).collect(),
      translucency: sums.iter().map(|(_, sum, count)| mean(*sum, *count)).collect(),
      positions: normals.positions,
      normals: normals.normals,
      groups,
    })
  }

  /// The ray model a model lights itself through: its shadow-casting faces in its own space, every one kept
  /// (`export_cform_rcast` with no transform).
  fn get_model_tracer(model: &BuildPrjMuModel, shaders: &CompilerShaders) -> XrlcTracer {
    let corner = |index: u32| {
      let vertex: &Vector3d = &model.vertices[index as usize];

      Vec3::new(vertex.x, vertex.y, vertex.z)
    };
    let triangles: Vec<Triangle> = model
      .faces
      .iter()
      .enumerate()
      .filter(|(number, face)| {
        shaders
          .require(model, *number, face)
          .is_ok_and(|shader| shader.casts_shadow())
      })
      .map(|(_, face)| Triangle::new(face.vertices.map(corner)))
      .collect();

    XrlcTracer::of_triangles(triangles)
  }

  /// `xrMU_Model::calc_lighting`: each vertex of the pass's model placed and lit as the pass says, each face a ray
  /// meets stopping what `opacity` says of the hit, the sky with its ambient; then the translucency pass and the
  /// halving.
  fn light(pass: &LightPass<'_>, opacity: impl Fn(&TraceHit) -> f32 + Sync) -> Vec<LightColor> {
    let LightPass {
      model,
      transform,
      lights,
      roles,
      samples,
      seed,
      tracer,
    } = *pass;
    let light_point: LightPoint = LightPoint {
      variant: LightPointVariant::Level,
    };
    let has_sky: bool = roles.contains(&LevelLightRole::Hemi);
    let colors: Vec<LightColor> = (0..model.positions.len())
      .into_par_iter()
      .map(|vertex| {
        let place: Vector3d = GeometryPlacement::transform(transform, &model.positions[vertex]);
        let turned: Vector3d = Self::rotate(transform, &model.normals[vertex]);
        let normal: Vector3d = turned.exact_normalize();
        let axis: Vec3 = Vec3::new(normal.x, normal.y, normal.z);
        let mut sum: LightColor = LightColor::default();

        for sample in 0..samples {
          let reach: f32 = Self::SAMPLE_REACH * sample as f32 / samples as f32;
          let jittered: Vec3 = Self::jitter(axis, seed | ((vertex as u64) << 8) | u64::from(sample));
          let point: Vec3 = Vec3::new(place.x, place.y, place.z) + jittered * reach;
          let lit: LightColor =
            light_point.evaluate_by(&LightSamplePoint::new(point, jittered), lights, roles, tracer, &opacity);

          sum = LightColor {
            rgb: sum.rgb + lit.rgb,
            hemi: sum.hemi + lit.hemi,
            sun: sum.sun + lit.sun,
          };
        }

        let mut color: LightColor = sum.get_scaled(1.0 / samples as f32);

        if has_sky {
          color.hemi += model.ambient[vertex];
        }

        color.get_stored()
      })
      .collect();
    let mut brightest: HashMap<u32, LightColor> = HashMap::new();

    for (color, group) in colors.iter().zip(&model.groups) {
      let entry: &mut LightColor = brightest.entry(*group).or_default();

      *entry = LightColor {
        rgb: entry.rgb.max(color.rgb),
        hemi: entry.hemi.max(color.hemi),
        sun: entry.sun.max(color.sun),
      };
    }

    colors
      .iter()
      .zip(&model.groups)
      .zip(&model.translucency)
      .map(|((color, group), translucency)| {
        let level: f32 = LightColor::get_fixed(*translucency);
        let other: &LightColor = &brightest[group];
        let unlevel: f32 = 1.0 - level;

        LightColor {
          rgb: color.rgb.lerp(other.rgb, level).max(color.rgb),
          hemi: (unlevel * color.hemi + level * other.hemi).max(color.hemi),
          sun: (unlevel * color.sun + level * other.sun).max(color.sun),
        }
        .get_scaled(0.5)
        .get_stored()
      })
      .collect()
  }

  /// A normal turned by `transform`'s rotation, its translation dropped (the inverse transpose of the rotation by
  /// `transform_dir`; the rotation of a placement is orthonormal, so the transform's own 3 by 3 part).
  fn rotate(transform: &Matrix4x4, normal: &Vector3d) -> Vector3d {
    let m: &[f32; Matrix4x4::ELEMENTS] = &transform.values;

    Vector3d::new(
      normal.x * m[0] + normal.y * m[4] + normal.z * m[8],
      normal.x * m[1] + normal.y * m[5] + normal.z * m[9],
      normal.x * m[2] + normal.y * m[6] + normal.z * m[10],
    )
  }

  /// A direction within [`Self::JITTER_DEGREES`] of `axis`, uniform over the cone, from `seed`.
  fn jitter(axis: Vec3, seed: u64) -> Vec3 {
    let mut state: u64 = seed;
    let mut next = || {
      // SplitMix64.
      state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);

      let mut value: u64 = state;

      value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
      value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
      ((value ^ (value >> 31)) >> 40) as f32 / (1u64 << 24) as f32
    };
    let cosine: f32 = 1.0 - next() * (1.0 - (Self::JITTER_DEGREES * PI / 180.0).cos());
    let sine: f32 = (1.0 - cosine * cosine).max(0.0).sqrt();
    let turn: f32 = next() * TAU;
    let (side, up) = axis.any_orthonormal_pair();

    (axis * cosine + side * (sine * turn.cos()) + up * (sine * turn.sin())).normalize()
  }

  /// `xrMU_Reference::calc_lighting`'s fit: the placement lit `colors`, its tree's scale and bias for each of red,
  /// green, blue, sky and sun the placement's term on the model's sky term.
  fn fit(model: &[LightColor], colors: Vec<LightColor>) -> LightReference {
    let terms = |color: &LightColor| [color.rgb.x, color.rgb.y, color.rgb.z, color.hemi, color.sun];
    let sky: Vec<f64> = model.iter().map(|color| f64::from(color.hemi)).collect();
    let model_sun: Vec<f64> = model.iter().map(|color| f64::from(color.sun)).collect();
    let mut scale: [f32; 5] = [1.0, 1.0, 1.0, 1.0, 1.0];
    let mut bias: [f32; 5] = [0.0; 5];

    for term in 0..5 {
      let values: Vec<f64> = colors.iter().map(|color| f64::from(terms(color)[term])).collect();

      (scale[term], bias[term]) = Self::regress(&sky, &values);
      (scale[term], bias[term]) = Self::optimize(&model_sun, &values, scale[term], bias[term]);
    }

    let color = |[red, green, blue, hemi, sun]: [f32; 5]| LightTreeColor {
      rgb: Vec3::new(red, green, blue),
      hemi,
      sun,
    };

    LightReference {
      colors,
      scale: color(scale),
      bias: color(bias),
    }
  }

  /// `vfComputeLinearRegression`: the least-squares line `b = a * scale + bias`, in double precision.
  pub(crate) fn regress(a: &[f64], b: &[f64]) -> (f32, f32) {
    let count: f64 = a.len() as f64;
    let sum_a: f64 = a.iter().sum();
    let sum_b: f64 = b.iter().sum();
    let sum_ab: f64 = a.iter().zip(b).map(|(a, b)| a * b).sum();
    let sum_aa: f64 = a.iter().map(|a| a * a).sum();
    let denominator: f64 = count * sum_aa - sum_a * sum_a;
    let scale: f32 = if denominator.abs() > f64::from(xrf_math::EPS_S) {
      ((count * sum_ab - sum_a * sum_b) / denominator) as f32
    } else {
      0.0
    };
    let bias: f32 = if a.is_empty() {
      0.0
    } else {
      ((sum_b - f64::from(scale) * sum_a) / count) as f32
    };

    (scale, bias)
  }

  /// `simple_optimize`: the line moved a seventh at a time toward the mean of `b` over `a`, while its mean error falls.
  pub(crate) fn optimize(a: &[f64], b: &[f64], start_scale: f32, start_bias: f32) -> (f32, f32) {
    let count: f64 = a.len() as f64;
    let (mut scale, mut bias): (f64, f64) = (f64::from(start_scale), f64::from(start_bias));
    let mut error: f64 = f64::from(f32::MAX);

    for _ in 0..128 {
      let (old_scale, old_bias): (f64, f64) = (scale, bias);
      let near: Vec<(f64, f64)> = a
        .iter()
        .zip(b)
        .filter(|(a, _)| a.abs() > f64::from(xrf_math::EPS_L))
        .map(|(a, b)| (*a, *b))
        .collect();
      let new_scale: f64 = if near.is_empty() {
        scale
      } else {
        near.iter().map(|(a, b)| (b - bias) / a).sum::<f64>() / near.len() as f64
      };
      let new_bias: f64 = if scale.abs() > f64::from(xrf_math::EPS) {
        a.iter().zip(b).map(|(a, b)| b - a / scale).sum::<f64>() / count
      } else {
        bias
      };

      scale = (6.0 * scale + new_scale) / 7.0;
      bias = (6.0 * bias + new_bias) / 7.0;

      let mean_error: f64 = a.iter().zip(b).map(|(a, b)| b - (a * scale + bias)).sum::<f64>() / count;

      if mean_error < error {
        error = mean_error;

        if error < f64::from(xrf_math::EPS) {
          return (scale as f32, bias as f32);
        }
      } else {
        return (old_scale as f32, old_bias as f32);
      }
    }

    (scale as f32, bias as f32)
  }
}
