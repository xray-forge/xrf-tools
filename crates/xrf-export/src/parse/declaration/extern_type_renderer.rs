use xrf_typescript::ast::canonical_ts_type_to_string;
use xrf_typescript::swc_common::SourceMap;
use xrf_typescript::swc_ecma_ast::{TsType, TsTypeAnn};

/// Writes a type annotation the way the manifest records it.
pub(crate) struct ExternTypeRenderer;

impl ExternTypeRenderer {
  /// What the manifest records for a type it has none for, or one it cannot render.
  pub const UNKNOWN: &'static str = "unknown";

  /// A type annotation in canonical form, `unknown` for syntax the renderer does not support.
  pub fn render(type_annotation: &TsType, source_map: &SourceMap) -> String {
    canonical_ts_type_to_string(type_annotation, source_map).unwrap_or_else(|_| String::from(Self::UNKNOWN))
  }

  /// An optional annotation in canonical form, `unknown` where it is not written.
  pub fn render_annotation(annotation: Option<&TsTypeAnn>, source_map: &SourceMap) -> String {
    annotation.map_or_else(
      || String::from(Self::UNKNOWN),
      |annotation| Self::render(annotation.type_ann.as_ref(), source_map),
    )
  }
}
