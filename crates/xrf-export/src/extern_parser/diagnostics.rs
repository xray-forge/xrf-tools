use xrf_error::XrfError;
use xrf_typescript::swc_common::{BytePos, SourceMap, SourceMapper, Span, Spanned};
use xrf_typescript::swc_ecma_ast::Expr;

use crate::extern_source_location::ExternSourceLocation;

/// Return an extern declaration source location, spanning a single line.
pub fn source_location(source_map: &SourceMap, position: BytePos, source_path: &str) -> ExternSourceLocation {
  let location = source_map.lookup_char_pos(position);

  ExternSourceLocation {
    column: location.col.0 + 1,
    line: location.line,
    end_line: location.line,
    path: source_path.into(),
  }
}

/// Return an extern declaration source location covering every line of `span`.
pub fn source_span_location(source_map: &SourceMap, span: Span, source_path: &str) -> ExternSourceLocation {
  ExternSourceLocation {
    end_line: source_map.lookup_char_pos(span.hi).line,
    ..source_location(source_map, span.lo, source_path)
  }
}

/// Create a location-aware invalid extern declaration error.
pub fn invalid_at(source_map: &SourceMap, position: BytePos, source_path: &str, reason: &str) -> XrfError {
  let location: ExternSourceLocation = source_location(source_map, position, source_path);

  XrfError::new_invalid_error(format!(
    "Invalid extern declaration at {}:{}:{}: {}.",
    location.path, location.line, location.column, reason,
  ))
}

/// Explain why an expression cannot provide a stable extern contract.
pub fn unsupported_export_value_reason(value: &Expr, export_name: &str, source_map: &SourceMap) -> String {
  let expression: String = source_map
    .span_to_snippet(value.span())
    .unwrap_or_else(|_| String::from("<unavailable>"));

  match value {
    Expr::Ident(identifier) => format!(
      "function reference `{}` for extern '{export_name}' needs a type; write `{expression} as (arg: Type) => ReturnType` or wrap it in an arrow function",
      identifier.sym,
    ),
    _ => format!(
      "extern '{export_name}' uses unsupported expression `{expression}`; use an arrow function or a `value as Type` assertion",
    ),
  }
}
