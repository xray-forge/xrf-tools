//! The detail lighter's shadow rays and the AI map's corner rays traced on the GPU, on Vulkan and on D3D12 through the
//! checkout's DXC, against the CPU's answers. A backend no GPU of the host traces rays on is skipped.

use std::path::PathBuf;

use glam::Vec3;
use xrf_error::XrfResult;
use xrf_geometry::{BvhTracer, Ray};
use xrf_renderer_core::{RayQueryAsk, RayQueryDevice};
use xrf_test_utils::TestSkip;

use crate::ai::cover::ai_cover_pass::AiNodeCovers;
use crate::detail::gpu::detail_gpu_batch::DetailGpuBatch;
use crate::detail::gpu::detail_gpu_records::GpuSingle;
use crate::detail::gpu::detail_gpu_tracer::DetailGpuTracer;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::tests::ai_covers;
use crate::tests::detail::{lighting, lighting_on, tracer};
use crate::{DetailLighting, RayBackend};

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

    let gpu: DetailGpuTracer = DetailGpuTracer::new(device, &tracer, &ShadowMaterials::new_opaque(4))?;

    let batch: DetailGpuBatch = DetailGpuBatch {
      singles: rays
        .iter()
        .zip(0..)
        .map(|(ray, index)| GpuSingle::new(ray, index))
        .collect(),
      ray_count: rays.len(),
      ..DetailGpuBatch::default()
    };

    assert_eq!(gpu.trace(&batch)?, [0.0, 1.0, 0.0, 1.0], "{name}");
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
