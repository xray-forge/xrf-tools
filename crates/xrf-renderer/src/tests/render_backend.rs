use crate::contract::render_backend::RenderBackend;

// The backend asked for is tried first, then every other once in the fallback order; none asked tries them all in it.
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
  assert_eq!(RenderBackend::get_preferred(None), RenderBackend::ALL[0]);
  assert_eq!(
    RenderBackend::get_preferred(Some(RenderBackend::Vulkan)),
    RenderBackend::Vulkan
  );

  if std::env::var_os(RenderBackend::ENVIRONMENT).is_none() {
    assert_eq!(
      RenderBackend::get_asked(Some(RenderBackend::Vulkan)),
      Some(RenderBackend::Vulkan)
    );
    assert_eq!(RenderBackend::get_asked(None), None);
  }
}
