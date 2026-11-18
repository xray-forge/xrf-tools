//! The detail lighter's shadow rays, the lightmaps' and the AI map's corner rays traced on the GPU, on Vulkan and on
//! D3D12 through the checkout's DXC, against the CPU's answers. A backend no GPU of the host traces rays on is skipped.

use std::path::PathBuf;

use glam::Vec3;
use xrf_error::XrfResult;
use xrf_geometry::{BvhTracer, Ray};
use xrf_renderer_core::{RayQueryAsk, RayQueryDevice};
use xrf_test_utils::TestSkip;

use crate::RayBackend;
use crate::ai::cover::ai_node_covers::AiNodeCovers;
use crate::detail::detail_lighting::DetailLighting;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::light::lightmap::lightmap_backend::LightmapBackend;
use crate::light::lightmap::lit::lit_chart::LitChart;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;
use crate::ray::gpu::ray_gpu_single::RayGpuSingle;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;
use crate::ray::ray_rules::RayRules;
use crate::ray::ray_tracing::RayTracing;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::tests::ai::ai_covers;
use crate::tests::detail::detail_lighter::{lighting, lighting_on, tracer};
use crate::tests::light::charts;

/// The DXC each backend is asked with: none, for Vulkan, then the checkout's, for D3D12 first.
fn dxcs() -> [Option<PathBuf>; 2] {
  [
    None,
    Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.sdks/dxc/dxcompiler.dll")),
  ]
}

/// A device tracing rays with `dxc`, or none where the host has none, the skip reported.
fn device(dxc: Option<&PathBuf>) -> Option<RayQueryDevice> {
  RayQueryDevice::request(RayQueryAsk {
    dxc: dxc.map(PathBuf::as_path),
    gpu: None,
  })
  .inspect_err(|error| TestSkip::report(format!("no GPU traces rays: {error}")))
  .ok()
}

#[test]
fn stops_a_ray_at_a_face_seen_from_its_front_alone() -> XrfResult {
  let tracer: BvhTracer = tracer();
  let rays: [Ray; 4] = [
    // Down onto the ground's top, its front.
    Ray::new(Vec3::new(0.5, 2.0, 1.0), Vec3::NEG_Y, 10.0),
    // Up through the ground from below, its back, then into the open beside the roof.
    Ray::new(Vec3::new(0.5, 0.0, 1.0), Vec3::Y, 10.0),
    // Up onto the roof's underside, its front.
    Ray::new(Vec3::new(3.0, 1.0, 1.0), Vec3::Y, 10.0),
    // Down toward the ground, ending short of it.
    Ray::new(Vec3::new(0.5, 2.0, 1.0), Vec3::NEG_Y, 1.0),
  ];

  for dxc in dxcs() {
    let Some(device) = device(dxc.as_ref()) else {
      continue;
    };
    let name: String = device.name.clone();

    // Where the checkout holds DXC, the device asked with it traces on D3D12 rather than falling back.
    if dxc.as_ref().is_some_and(|dxc| dxc.is_file()) {
      assert!(name.ends_with("(D3D12)"), "{name}");
    }

    let gpu: RayGpuTracer = RayGpuTracer::new(device, &tracer, &ShadowMaterials::new_opaque(4))?;
    let batch = |skip: u32| RayGpuBatch {
      singles: rays
        .iter()
        .zip(0..)
        .map(|(ray, index)| RayGpuSingle::new(ray, index, skip))
        .collect(),
      ray_count: rays.len(),
      ..RayGpuBatch::default()
    };

    assert_eq!(
      gpu.trace(&batch(RayGpuTracer::NO_SKIP), RayRules::DETAILS)?,
      [0.0, 1.0, 0.0, 1.0],
      "{name}"
    );
    // By xrLC's rules a face stops a ray from either side: the ground's back too.
    assert_eq!(
      gpu.trace(&batch(RayGpuTracer::NO_SKIP), RayRules::LEVEL)?,
      [0.0, 0.0, 0.0, 1.0],
      "{name}"
    );
  }

  Ok(())
}

#[test]
fn passes_a_ray_through_its_own_face_by_xrlc_s_rules() -> XrfResult {
  let tracer: BvhTracer = tracer();
  // Down onto the ground's top, which is face 0 or 1; the ground stops it unless it passes the face it meets.
  let ray: Ray = Ray::new(Vec3::new(0.5, 2.0, 1.0), Vec3::NEG_Y, 10.0);
  let mut met: Option<u32> = None;

  xrf_geometry::Tracer::for_each_hit(&tracer, &ray, |hit| met = met.or(Some(hit.face)));

  let met: u32 = met.expect("the ray meets the ground");

  for dxc in dxcs() {
    let Some(device) = device(dxc.as_ref()) else {
      continue;
    };
    let name: String = device.name.clone();
    let gpu: RayGpuTracer = RayGpuTracer::new(device, &tracer, &ShadowMaterials::new_opaque(4))?;
    let batch = |skip: u32| RayGpuBatch {
      singles: vec![RayGpuSingle::new(&ray, 0, skip)],
      ray_count: 1,
      ..RayGpuBatch::default()
    };

    assert_eq!(
      gpu.trace(&batch(gpu.get_skip(Some(met))), RayRules::LEVEL)?,
      [1.0],
      "{name}"
    );
    assert_eq!(
      gpu.trace(&batch(RayGpuTracer::NO_SKIP), RayRules::LEVEL)?,
      [0.0],
      "{name}"
    );
    // The detail lighter's rules pass no face.
    assert_eq!(
      gpu.trace(&batch(gpu.get_skip(Some(met))), RayRules::DETAILS)?,
      [0.0],
      "{name}"
    );
  }

  Ok(())
}

#[test]
fn lights_a_chart_by_a_lamp_and_the_sun_as_the_cpu_does() -> XrfResult {
  let mesh: GeometryMesh = charts::floor_and_wall(0.0);
  // A lamp 6 m over the wall's foot, 1 m behind it: the wall's top shades the floor's far half from it.
  let lamp: LightSource = LightSource {
    is_directional: false,
    is_point: true,
    position: Vec3::new(1.0, 6.0, 3.0),
    direction: Vec3::NEG_Y,
    range: 10.0,
    range_squared: 100.0,
    ..charts::sun(1.0)
  };
  let lights: LightSources = LightSources {
    rgb: vec![lamp],
    hemi: Vec::new(),
    sun: vec![charts::sun(0.8)],
  };
  let on_cpu: LitChart = charts::light_by(&mesh, &lights, |lighter, floor| lighter.light(floor));

  assert!(
    on_cpu
      .layer
      .surface
      .iter()
      .any(|texel| *texel != on_cpu.layer.surface[0]),
    "the lamp's shadow leaves the chart more than one colour"
  );

  for dxc in dxcs() {
    if device(dxc.as_ref()).is_none() {
      continue;
    }

    let tracing: RayTracing<'_> = RayTracing {
      backend: RayBackend::Gpu,
      dxc: dxc.as_deref(),
      gpu: None,
    };
    let on_gpu: Vec<LitChart> = charts::light_by(&mesh, &lights, |lighter, floor| {
      LightmapBackend::start(&tracing, lighter)?.light_charts(
        lighter,
        std::slice::from_ref(floor),
        &xrf_job::JobHandle::inert(),
      )
    })?;

    assert_eq!(on_gpu, std::slice::from_ref(&on_cpu), "{dxc:?}");
  }

  Ok(())
}

#[test]
fn lights_the_slots_as_the_cpu_does() -> XrfResult {
  for dxc in dxcs() {
    if device(dxc.as_ref()).is_none() {
      continue;
    }

    let on_gpu: DetailLighting = lighting_on(RayBackend::Gpu, dxc.as_deref())?;
    let name: String = on_gpu.gpu.clone().unwrap_or_default();

    assert_eq!(on_gpu.details, lighting()?.details, "{name}");
    assert!(on_gpu.gpu.is_some());
  }

  Ok(())
}

#[test]
fn covers_the_ai_map_as_the_cpu_does() -> XrfResult {
  let tracer: BvhTracer = ai_covers::wall();
  let on_cpu: Vec<AiNodeCovers> = ai_covers::covers(&tracer, RayBackend::Cpu, None)?;

  for dxc in dxcs() {
    if device(dxc.as_ref()).is_none() {
      continue;
    }

    assert_eq!(
      ai_covers::covers(&tracer, RayBackend::Gpu, dxc.as_deref())?,
      on_cpu,
      "{dxc:?}"
    );
  }

  Ok(())
}
