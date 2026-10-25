use serde::Serialize;
use specta::datatype::DataType;
use specta::{Type, Types};

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

  /// A constant's value as it serializes, which is what the frontend's bindings declare it as; an error for one that
  /// does not serialize to JSON, as a map with non-string keys does not.
  #[doc(hidden)]
  pub fn serialize_value<T: Serialize>(value: &T) -> serde_json::Result<serde_json::Value> {
    // Through the text rather than `serde_json::to_value`, which widens an `f32` to `f64` and so gives `0.1` as
    // `0.10000000149011612`: the serializer writes an `f32` in its own shortest form.
    serde_json::from_str(&serde_json::to_string(value)?)
  }

  /// A constant's type as Specta defines it, read off the value so the expansion names no type the marked item spelled;
  /// registers every named type that one references into `types`.
  #[doc(hidden)]
  pub fn define_type<T: Type + ?Sized>(_value: &T, types: &mut Types) -> DataType {
    T::definition(types)
  }

  /// The module enclosing the one `module_path!()` names at `path`: `path` without its last segment, itself at a root.
  /// The expansion asks it from inside the hidden registration module, so this is the module declaring the constant.
  #[doc(hidden)]
  pub const fn enclosing_module(path: &'static str) -> &'static str {
    let bytes: &[u8] = path.as_bytes();
    let mut end: usize = bytes.len();

    while end >= 2 {
      if bytes[end - 2] == b':' && bytes[end - 1] == b':' {
        return path.split_at(end - 2).0;
      }

      end -= 1;
    }

    path
  }
}
