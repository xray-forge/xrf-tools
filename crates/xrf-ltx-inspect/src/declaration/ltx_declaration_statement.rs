use serde::Serialize;

/// The statement a declaration search matched: a section header, or one field line.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LtxDeclarationStatement {
  /// A section header, with the parents it names.
  Section { parents: Vec<String> },
  /// A field line, with its own operation prefix - empty, `!`, `>` or `<` - and its value as written.
  ///
  /// `value` is absent for a bare key, which is not the same as an empty value.
  Field {
    name: String,
    operation: String,
    value: Option<String>,
  },
}
