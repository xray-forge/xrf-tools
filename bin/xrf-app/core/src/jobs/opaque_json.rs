/// The Typescript face of a value this application does not describe: `unknown`.
///
/// `serde_json::Value` cannot be mirrored directly - Specta inlines it, and a self-referential inline type has no
/// finite Typescript expansion. Naming it `unknown` says the same thing the Rust side says, and better than a shape
/// would: whoever reads the field is the one that knows what is in it, and a caller that guesses has to narrow first.
#[cfg(feature = "typescript-bindings")]
pub struct OpaqueJson;

#[cfg(feature = "typescript-bindings")]
impl specta::Type for OpaqueJson {
  fn definition(_: &mut specta::Types) -> specta::datatype::DataType {
    specta::datatype::DataType::Reference(specta_typescript::define("unknown"))
  }
}
