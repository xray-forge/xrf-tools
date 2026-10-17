/// A constant the frontend's bindings export, registered where it is declared by `#[xrf_ipc::constant]`.
pub struct IpcConstant {
  /// `module_path!()` where it is declared, whose first segment names the crate owning it.
  pub module_path: &'static str,
  pub name: &'static str,
  /// Its doc comment, its lines joined by `\n`.
  pub doc: &'static str,
  /// Its value as it serializes, or why it does not.
  pub value: fn() -> serde_json::Result<serde_json::Value>,
}

inventory::collect!(IpcConstant);

impl IpcConstant {
  /// Every constant registered in the binary, in no particular order.
  pub fn list() -> impl Iterator<Item = &'static IpcConstant> {
    inventory::iter::<IpcConstant>.into_iter()
  }
}
