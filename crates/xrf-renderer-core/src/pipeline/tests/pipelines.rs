use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};

use crate::{ComputePipelineDescription, PipelineCache, RenderPipelineDescription, ShaderPermutation, ShaderSource};

#[derive(Clone, Copy, crate::ShaderPermutation)]
struct Shadowing {
  is_shadow: bool,
  cascade: u32,
}

/// Modules by name, written out whole.
struct TestSource(HashMap<&'static str, String>);

impl ShaderSource for TestSource {
  fn compose(&self, module: &str) -> XrfResult<String> {
    self
      .0
      .get(module)
      .cloned()
      .ok_or_else(|| XrfError::new_not_found_error(format!("No module '{module}'")))
  }
}

fn source() -> TestSource {
  TestSource(HashMap::from([
    (
      "count",
      format!(
        "{}@group(0) @binding(0) var<storage, read_write> counts: array<u32>;
         @compute @workgroup_size(1) fn main() {{ counts[cascade] = select(1u, 2u, is_shadow); }}",
        Shadowing::get_wgsl_overrides()
      ),
    ),
    (
      "fill",
      "@vertex fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
         return vec4<f32>(f32(index & 1u) * 4.0 - 1.0, f32(index >> 1u) * 4.0 - 1.0, 0.0, 1.0);
       }
       @fragment fn fragment() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }"
        .to_string(),
    ),
    ("broken", "fn main( {".to_string()),
  ]))
}

#[test]
fn derives_the_constants_and_their_overrides() {
  let shadowing: Shadowing = Shadowing {
    is_shadow: true,
    cascade: 2,
  };

  assert_eq!(shadowing.list_constants(), [("is_shadow", 1.0), ("cascade", 2.0)]);
  assert_eq!(
    Shadowing::get_wgsl_overrides(),
    "override is_shadow: bool;\noverride cascade: u32;\n"
  );

  let code: String = source().compose("count").unwrap();
  let module: naga::Module = naga::front::wgsl::parse_str(&code).unwrap();

  naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
    .validate(&module)
    .unwrap();
}

#[test]
fn makes_a_pipeline_once_per_permutation_and_forgets_an_invalidated_module() {
  let Some((device, _queue)) = crate::tests::test_device::create_device() else {
    return;
  };
  let cache: PipelineCache = PipelineCache::new();
  let source: TestSource = source();
  let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("counts"),
    entries: &[crate::PassBindingLayout::storage(0, true)],
  });
  let describe = |is_shadow: bool| {
    ComputePipelineDescription::new("count", "count", "main")
      .with_permutation(&Shadowing { is_shadow, cascade: 1 })
      .with_bind_group_layouts(vec![layout.clone()])
  };

  cache.get_compute_pipeline(&device, &source, &describe(false)).unwrap();
  cache.get_compute_pipeline(&device, &source, &describe(true)).unwrap();
  cache.get_compute_pipeline(&device, &source, &describe(true)).unwrap();

  assert_eq!(cache.get_pipeline_count(), 2, "one pipeline per permutation");

  cache.invalidate_module("count");

  assert_eq!(cache.get_pipeline_count(), 0);

  cache.get_compute_pipeline(&device, &source, &describe(true)).unwrap();

  assert_eq!(cache.get_pipeline_count(), 1);
}

#[test]
fn makes_a_render_pipeline_and_warms_ahead() {
  let Some((device, _queue)) = crate::tests::test_device::create_device() else {
    return;
  };
  let cache: PipelineCache = PipelineCache::new();
  let description: RenderPipelineDescription = RenderPipelineDescription::new("fill", "fill", "vertex")
    .with_fragment("fragment")
    .with_target(wgpu::ColorTargetState::from(wgpu::TextureFormat::Rgba8Unorm));

  cache
    .warm(&device, &source(), std::slice::from_ref(&description), &[])
    .unwrap();
  cache.get_render_pipeline(&device, &source(), &description).unwrap();

  assert_eq!(
    cache.get_pipeline_count(),
    1,
    "warming made the pipeline the frame then asked for"
  );
}

#[test]
fn refuses_a_module_wgpu_cannot_compile() {
  let Some((device, _queue)) = crate::tests::test_device::create_device() else {
    return;
  };
  let error: String = PipelineCache::new()
    .get_compute_pipeline(
      &device,
      &source(),
      &ComputePipelineDescription::new("broken", "broken", "main"),
    )
    .unwrap_err()
    .to_string();

  assert!(error.contains("'broken'"), "{error}");
}
