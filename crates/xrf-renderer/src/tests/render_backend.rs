use xrf_env_flags::EnvFlags;

use crate::contract::render_backend::RenderBackend;

// The backend asked for is tried first, then every other once in the fallback order; none asked tries them all in it.
#[cfg(not(target_vendor = "apple"))]
#[test]
fn tries_the_asked_backend_first_then_the_rest() {
  assert_eq!(RenderBackend::list_tried(None), RenderBackend::ALL.to_vec());
  assert_eq!(
    RenderBackend::list_tried(Some(RenderBackend::Vulkan)),
    vec![RenderBackend::Vulkan, RenderBackend::D3d12]
  );
  assert_eq!(
    RenderBackend::list_tried(Some(RenderBackend::D3d12)),
    vec![RenderBackend::D3d12, RenderBackend::Vulkan]
  );
}

// A backend this platform does not build is never tried, wherever it was asked for: the platform's own start instead.
#[test]
fn tries_only_the_backends_this_platform_builds() {
  for asked in [RenderBackend::D3d12, RenderBackend::Vulkan, RenderBackend::Metal] {
    let tried: Vec<RenderBackend> = RenderBackend::list_tried(Some(asked));

    assert!(tried.iter().all(|it| it.is_built()), "{asked:?} tried {tried:?}");
    assert_eq!(tried.len(), RenderBackend::ALL.len());
    assert_eq!(tried[0], if asked.is_built() { asked } else { RenderBackend::ALL[0] });
  }

  assert_eq!(RenderBackend::default(), RenderBackend::ALL[0]);
}

#[cfg(target_vendor = "apple")]
#[test]
fn draws_with_metal_alone_on_apple_platforms() {
  assert_eq!(RenderBackend::ALL, &[RenderBackend::Metal]);
}

// The settings name a backend as the UI does.
#[test]
fn names_backends_as_the_settings_do() {
  assert_eq!(serde_json::to_string(&RenderBackend::D3d12).unwrap(), "\"d3d12\"");
  assert_eq!(
    serde_json::from_str::<RenderBackend>("\"vulkan\"").unwrap(),
    RenderBackend::Vulkan
  );
}

// The GPU starts on the backend asked for, else on the first; the environment's overrides the settings' only when set.
#[test]
fn prefers_the_asked_backend_else_the_first() {
  let last: RenderBackend = *RenderBackend::ALL.last().expect("every platform builds a backend");

  assert_eq!(RenderBackend::get_preferred(None), RenderBackend::ALL[0]);
  assert_eq!(RenderBackend::get_preferred(Some(last)), last);

  if EnvFlags::RENDER_BACKEND.get_raw().is_none() {
    assert_eq!(
      RenderBackend::get_asked(Some(RenderBackend::Vulkan)),
      Some(RenderBackend::Vulkan)
    );
    assert_eq!(RenderBackend::get_asked(None), None);
  }
}
