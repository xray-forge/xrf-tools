// A value Rust cannot write as a `const` is computed by a function of no arguments, exported in upper snake case.
#[xrf_ipc::constant]
fn render_shadow_cascade_widths() -> Vec<f32> {
  vec![20.0, 40.0, 160.0, 480.0]
}

fn main() {
  let exported = xrf_ipc::IpcConstant::list()
    .find(|it| it.name == "RENDER_SHADOW_CASCADE_WIDTHS")
    .expect("registered");

  assert_eq!(
    (exported.value)().expect("serialized"),
    xrf_ipc::serde_json::json!(render_shadow_cascade_widths())
  );
}
