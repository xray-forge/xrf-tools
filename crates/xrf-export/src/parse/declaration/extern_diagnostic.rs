use xrf_error::XrfError;
use xrf_typescript::swc_common::{BytePos, SourceMap, SourceMapper, Span, Spanned};
use xrf_typescript::swc_ecma_ast::Expr;

use crate::parse::extern_source_location::ExternSourceLocation;

/// Places what one source's declarations say, and words the errors raised about them.
#[derive(Clone, Copy)]
pub(crate) struct ExternDiagnostic<'a> {
  source_map: &'a SourceMap,
  source_path: &'a str,
}

impl<'a> ExternDiagnostic<'a> {
  pub fn new(source_map: &'a SourceMap, source_path: &'a str) -> Self {
    Self {
      source_map,
      source_path,
    }
  }

  /// The source location of one position, spanning its line alone.
  pub fn location(&self, position: BytePos) -> ExternSourceLocation {
    let location = self.source_map.lookup_char_pos(position);

    ExternSourceLocation {
      column: location.col.0 + 1,
      line: location.line,
      end_line: location.line,
      path: self.source_path.into(),
    }
  }

  /// The source location of a span, covering every line of it.
  pub fn span_location(&self, span: Span) -> ExternSourceLocation {
    ExternSourceLocation {
      end_line: self.source_map.lookup_char_pos(span.hi).line,
      ..self.location(span.lo)
    }
  }

  /// A declaration that cannot be read, placed where it goes wrong.
  pub fn invalid_at(&self, position: BytePos, reason: &str) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Invalid extern declaration at {}: {reason}.",
      self.location(position)
    ))
  }

  /// A value no stable contract can be read from, with what to write instead.
  pub fn invalid_value(&self, value: &Expr, export_name: &str) -> XrfError {
    let expression: String = self
      .source_map
      .span_to_snippet(value.span())
      .unwrap_or_else(|_| String::from("<unavailable>"));
    let reason: String = match value {
      Expr::Ident(identifier) => format!(
        "function reference `{}` for extern '{export_name}' needs a type; write `{expression} as (arg: Type) => \
         ReturnType` or wrap it in an arrow function",
        identifier.sym,
      ),
      _ => format!(
        "extern '{export_name}' uses unsupported expression `{expression}`; use an arrow function or a `value as Type` \
         assertion",
      ),
    };

    self.invalid_at(value.span().lo, &reason)
  }

  pub fn get_source_map(&self) -> &'a SourceMap {
    self.source_map
  }

  pub fn get_source_path(&self) -> &'a str {
    self.source_path
  }
}
