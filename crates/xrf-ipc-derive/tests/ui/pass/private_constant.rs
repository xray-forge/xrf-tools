mod limits {
  /// Cascades the sun's shadow can be cut into at most:
  ///   the sun reads their texels from one `vec4`.
  #[xrf_ipc::constant]
  const RENDER_MAX_SHADOW_CASCADES: usize = 4;

  pub fn get() -> usize {
    RENDER_MAX_SHADOW_CASCADES
  }
}

fn main() {
  let exported = xrf_ipc::IpcConstant::list()
    .find(|it| it.name == "RENDER_MAX_SHADOW_CASCADES")
    .expect("registered");

  assert_eq!(limits::get(), 4);
  assert!(exported.module_path.ends_with("::limits"), "{}", exported.module_path);
  assert_eq!(
    exported.doc,
    "Cascades the sun's shadow can be cut into at most:\n  the sun reads their texels from one `vec4`."
  );
  assert_eq!((exported.value)(), xrf_ipc::serde_json::json!(4));
}
