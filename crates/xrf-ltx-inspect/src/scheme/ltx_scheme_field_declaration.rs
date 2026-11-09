use serde::Serialize;
use xrf_ltx::LtxFieldScheme;

/// What a scheme declares about one field.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxSchemeFieldDeclaration {
  /// The type as the scheme spells it - `u32`, `enum:pistol,rifle`, `condlist`.
  pub data_type: String,
  pub is_array: bool,
  pub is_optional: bool,
  /// Whether this is the scheme's catch-all `*` rather than a declaration naming the field.
  pub is_any: bool,
}

impl LtxSchemeFieldDeclaration {
  /// One declared field, as a record something outside Rust can render; `is_any` for the scheme's catch-all.
  pub(crate) fn of(declared: &LtxFieldScheme, is_any: bool) -> Self {
    Self {
      data_type: declared.data_type.to_string(),
      is_any,
      is_array: declared.is_array,
      is_optional: declared.is_optional,
    }
  }
}
