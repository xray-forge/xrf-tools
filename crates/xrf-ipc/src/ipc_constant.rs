use specta::Types;
use specta::datatype::DataType;

/// A constant the frontend's bindings export, registered where it is declared by `#[xrf_ipc::constant]`.
pub struct IpcConstant {
  /// `module_path!()` where it is declared, whose first segment names the crate owning it.
  pub module_path: &'static str,
  pub name: &'static str,
  /// Its doc comment, its lines joined by `\n`.
  pub doc: &'static str,
  /// Its value as it serializes, or why it does not.
  pub value: fn() -> serde_json::Result<serde_json::Value>,
  /// Its type's Specta definition, registering every named type that one references into the collection, so a type
  /// only a constant carries is still declared.
  pub data_type: fn(&mut Types) -> DataType,
}

inventory::collect!(IpcConstant);

impl IpcConstant {
  /// Every constant registered in the binary, in no particular order.
  pub fn list() -> impl Iterator<Item = &'static IpcConstant> {
    inventory::iter::<IpcConstant>.into_iter()
  }
}
