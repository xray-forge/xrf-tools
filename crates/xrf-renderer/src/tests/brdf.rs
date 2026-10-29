//! `common/brdf` on the GPU: its direct light held to the published terms written out here, and its lobe integrated
//! over the hemisphere, where a white surface under a uniform sky must reflect the sky whole; and the split-sum table
//! it takes the multiple scattering's gift from.

use std::f32::consts::PI;

use glam::{Vec2, Vec3, Vec4};

use crate::pass::brdf_table::BrdfTable;
use crate::tests::gpu_probe::run_gpu_probe;

/// The probe's view, about +z at the cosine `first.x`, and its direct light from `second.xyz` at roughness `first.y`,
/// reflectance head on `first.z` and white albedo `first.w`.
const DIRECT: &str = "let to_eye: vec3<f32> = vec3<f32>(sqrt(1.0 - first.x * first.x), 0.0, first.x);
  let light: DirectLight =
    direct_light(vec3<f32>(0.0, 0.0, 1.0), to_eye, second.xyz, SphereLight(second.xyz, 1.0), first.y,
      vec3<f32>(first.z), first.w);";

/// GGX's distribution, Smith's height-correlated visibility and Schlick's Fresnel for a grey reflectance head on, with
/// the multiple scattering's gift over the lobe's white albedo, as Walter et al., Heitz and Fdez-Agüera write them.
fn reference_specular(to_eye: Vec3, to_light: Vec3, roughness: f32, head_on: f32, white: f32) -> f32 {
  let lit: f32 = to_light.z;
  let facing: f32 = to_eye.z.max(1e-4);
  let half_way: Vec3 = (to_light + to_eye).normalize();
  let alpha: f32 = roughness * roughness;
  let alpha_squared: f32 = alpha * alpha;
  let along: f32 = half_way.z.clamp(0.0, 1.0);
  let distribution: f32 = alpha_squared / (PI * (along * along * (alpha_squared - 1.0) + 1.0).powi(2));
  let visibility: f32 = 0.5
    / (lit * (facing * facing * (1.0 - alpha_squared) + alpha_squared).sqrt()
      + facing * (lit * lit * (1.0 - alpha_squared) + alpha_squared).sqrt());
  let fresnel: f32 = head_on + (1.0 - head_on) * (1.0 - to_eye.dot(half_way).clamp(0.0, 1.0)).powi(5);
  let gift: f32 = 1.0 + head_on * (1.0 / white.max(1e-3) - 1.0);

  PI * distribution * visibility * fresnel * lit * gift
}

/// A direction about +z at a polar cosine and an azimuth.
fn direction(cosine: f32, azimuth: f32) -> Vec3 {
  let sine: f32 = (1.0 - cosine * cosine).max(0.0).sqrt();

  Vec3::new(sine * azimuth.cos(), sine * azimuth.sin(), cosine)
}

/// The lobe's directional albedo for white at a view's cosine and a roughness, from the split-sum table.
fn white(facing: f32, roughness: f32) -> f32 {
  let split: Vec2 = BrdfTable::integrate(facing, roughness);

  split.x + split.y
}

// The direct light's diffuse is the cosine and its specular GGX's lobe over Smith's height-correlated visibility and
// Schlick's Fresnel, given back what single scattering loses, as the terms are published, against +z.
#[test]
fn the_direct_light_is_the_published_lobe() {
  let mut cases: Vec<(Vec4, Vec4)> = Vec::new();
  let mut expected: Vec<Vec3> = Vec::new();

  for roughness in [0.05_f32, 0.3, 0.6, 1.0] {
    for head_on in [0.04_f32, 0.5, 1.0] {
      for (facing, light) in [(0.9, (0.9, 3.0)), (0.4, (0.7, 2.0)), (0.15, (0.3, 4.2))] {
        let to_light: Vec3 = direction(light.0, light.1);
        let white: f32 = white(facing, roughness);

        cases.push((Vec4::new(facing, roughness, head_on, white), to_light.extend(0.0)));
        expected.push(Vec3::new(
          light.0,
          reference_specular(direction(facing, 0.0), to_light, roughness, head_on, white),
          0.0,
        ));
      }
    }
  }

  let body: String = format!("{DIRECT}\n  answer = vec3<f32>(light.diffuse, light.specular.x, 0.0);");
  let Some(answers) = run_gpu_probe("common/brdf", &body, &cases) else {
    return;
  };

  for ((answer, expected), case) in answers.iter().zip(&expected).zip(&cases) {
    assert!(
      (*answer - *expected).abs().max_element() <= expected.abs().max_element() * 1e-3 + 1e-6,
      "{case:?}: {answer} against {expected}"
    );
  }
}

// Under a uniform sky of radiance one, which lights a surface from each solid angle `dω` as a light of colour `dω/π`
// would, a white metal's lobe reflects the sky whole at every roughness and view, the multiple scattering giving back
// what one scattering loses on a rough lobe, and Lambert's cosine likewise: the white furnace.
#[test]
fn a_white_surface_reflects_a_uniform_sky_whole() {
  let mut cases: Vec<(Vec4, Vec4)> = Vec::new();

  for roughness in [0.3_f32, 0.5, 0.7, 1.0] {
    for facing in [0.3_f32, 0.6, 1.0] {
      cases.push((Vec4::new(facing, roughness, 1.0, white(facing, roughness)), Vec4::ZERO));
    }
  }

  // The hemisphere in cells even in the polar cosine and the azimuth, each `2π / (N M)` of solid angle.
  let body: String = format!(
    "let steps: vec2<u32> = vec2<u32>(512u, 256u);
  let cell: f32 = 2.0 * PI / f32(steps.x * steps.y);
  var specular: f32 = 0.0;
  var diffuse: f32 = 0.0;

  for (var i: u32 = 0u; i < steps.x; i++) {{
    let cosine: f32 = (f32(i) + 0.5) / f32(steps.x);
    let sine: f32 = sqrt(1.0 - cosine * cosine);

    for (var j: u32 = 0u; j < steps.y; j++) {{
      let azimuth: f32 = 2.0 * PI * (f32(j) + 0.5) / f32(steps.y);
      let second: vec4<f32> = vec4<f32>(sine * cos(azimuth), sine * sin(azimuth), cosine, 0.0);
      {DIRECT}

      specular += light.specular.x * cell / PI;
      diffuse += light.diffuse * cell / PI;
    }}
  }}

  answer = vec3<f32>(specular, diffuse, 0.0);"
  );
  let Some(answers) = run_gpu_probe("common/brdf", &body, &cases) else {
    return;
  };
  let reflected: Vec<String> = answers
    .iter()
    .zip(&cases)
    .map(|(answer, (case, _))| {
      format!(
        "roughness {} facing {}: {:.3} {:.4}",
        case.y, case.x, answer.x, answer.y
      )
    })
    .collect();

  assert!(
    answers
      .iter()
      .all(|answer| (answer.x - 1.0).abs() < 0.03 && (answer.y - 1.0).abs() < 1e-3),
    "a white metal's lobe, then Lambert's, against one:\n{}",
    reflected.join("\n")
  );
}

// A sphere light taken from its nearest point gives about what its disc gives as the many point lights it is, each
// direction of its cone a share of its colour: the glint widened and held to the energy the sphere sends, at the
// reflection's peak and off it. Karis's approximation keeps a rough lobe within a quarter under the disc, and gives a
// smooth lobe whose reflection falls just past the rim up to two thirds over, the nearest point's lobe at its peak.
#[test]
fn a_sphere_light_glints_as_its_disc_does() {
  let mut cases: Vec<(Vec4, Vec4)> = Vec::new();

  for roughness in [0.2_f32, 0.35, 0.55] {
    for angular_radius in [0.05_f32, 0.15] {
      for offset in [0.0_f32, 0.1] {
        cases.push((
          Vec4::new(0.6, roughness, 0.04, white(0.6, roughness)),
          Vec4::new(angular_radius, offset, 0.0, 0.0),
        ));
      }
    }
  }

  // The view about +z at the cosine `first.x`; the sphere a metre away about its reflection, turned `second.y` from it,
  // its cone `second.x` wide, sampled in cells even in solid angle.
  let body: &str = "let to_eye: vec3<f32> = vec3<f32>(sqrt(1.0 - first.x * first.x), 0.0, first.x);
  let normal: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
  let reflected: vec3<f32> = reflect(-to_eye, normal);
  let turned: f32 = acos(first.x) + second.y;
  let centre: vec3<f32> = vec3<f32>(-sin(turned), 0.0, cos(turned));
  let across: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
  let along: vec3<f32> = cross(across, centre);
  let edge: f32 = cos(second.x);
  let steps: vec2<u32> = vec2<u32>(256u, 256u);
  var disc: f32 = 0.0;

  for (var i: u32 = 0u; i < steps.x; i++) {
    let cosine: f32 = mix(edge, 1.0, (f32(i) + 0.5) / f32(steps.x));
    let sine: f32 = sqrt(1.0 - cosine * cosine);

    for (var j: u32 = 0u; j < steps.y; j++) {
      let azimuth: f32 = 2.0 * PI * (f32(j) + 0.5) / f32(steps.y);
      let to_light: vec3<f32> = centre * cosine + (along * cos(azimuth) + across * sin(azimuth)) * sine;

      disc += direct_light(normal, to_eye, to_light, SphereLight(to_light, 1.0), first.y, vec3<f32>(first.z),
        first.w).specular.x;
    }
  }

  let sphere: SphereLight = sphere_light(centre, sin(second.x), reflected, first.y);
  let taken: f32 =
    direct_light(normal, to_eye, centre, sphere, first.y, vec3<f32>(first.z), first.w).specular.x;

  answer = vec3<f32>(disc / f32(steps.x * steps.y), taken, 0.0);";
  let Some(answers) = run_gpu_probe("common/brdf", body, &cases) else {
    return;
  };
  let compared: Vec<String> = answers
    .iter()
    .zip(&cases)
    .map(|(answer, (first, second))| {
      format!(
        "roughness {} radius {} offset {}: disc {:.4} sphere {:.4} ({:.2})",
        first.y,
        second.x,
        second.y,
        answer.x,
        answer.y,
        answer.y / answer.x
      )
    })
    .collect();

  assert!(
    answers
      .iter()
      .all(|answer| (0.75..1.7).contains(&(answer.y / answer.x))),
    "the sphere's nearest point against its disc:\n{}",
    compared.join("\n")
  );
}

// The table falls from a mirror's whole reflection towards a rough lobe's less, and grazing views reflect more of the
// Fresnel's share; every texel stays a share of the light.
#[test]
fn the_split_sum_table_holds_shares_of_the_light() {
  let table: Vec<[f32; 2]> = BrdfTable::create_table();

  assert_eq!(table.len(), (BrdfTable::SIZE * BrdfTable::SIZE) as usize);
  assert!(
    table
      .iter()
      .all(|[scale, bias]| *scale >= 0.0 && *bias >= 0.0 && scale + bias <= 1.0 + 1e-3)
  );
  assert!(white(1.0, 0.05) > 0.99);
  assert!(white(1.0, 1.0) < white(1.0, 0.3));
  assert!(BrdfTable::integrate(0.2, 0.3).y > BrdfTable::integrate(1.0, 0.3).y);
}
