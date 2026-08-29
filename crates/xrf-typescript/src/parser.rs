use std::default::Default;
use std::path::Path;
use std::rc::Rc;

use swc_common::comments::SingleThreadedComments;
use swc_common::errors::DiagnosticBuilder;
use swc_common::sync::Lrc;
use swc_common::{
  SourceFile, SourceMap,
  errors::{ColorConfig, Handler},
};
use swc_ecma_ast::Program;
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// A parsed TypeScript file together with the metadata needed for diagnostics.
pub struct TypeScriptSource {
  pub comments: SingleThreadedComments,
  pub program: Program,
  pub source_map: Lrc<SourceMap>,
}

/// Parse a TypeScript source file and collect its comments and source map.
///
/// The parser uses SWC's TypeScript syntax and returns an `XrfError` when
/// SWC reports parsing diagnostics before a program can be produced.
pub fn parse_typescript_file(path: &Path) -> XrfResult<TypeScriptSource> {
  let source_map: Lrc<SourceMap> = Default::default();
  let handler: Handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map.clone()));
  let source_file: Rc<SourceFile> = source_map
    .load_file(path)
    .expect("Failed to load TypeScript source file");
  let comments: SingleThreadedComments = Default::default();

  let lexer: Lexer = Lexer::new(
    Syntax::Typescript(Default::default()),
    Default::default(),
    StringInput::from(source_file.as_ref()),
    Some(&comments),
  );
  let mut parser: Parser<Lexer> = Parser::new_from(lexer);
  let mut diagnostics: Vec<DiagnosticBuilder> = parser
    .take_errors()
    .into_iter()
    .map(|it| it.into_diagnostic(&handler))
    .collect();

  for diagnostic in &mut diagnostics {
    diagnostic.emit();
  }

  if !diagnostics.is_empty() {
    return Err(XrfError::new_parsing_error(format!(
      "Failed to parse TypeScript file {}: {}",
      format_path(path),
      diagnostics
        .iter()
        .map(|builder| builder
          .message
          .iter()
          .map(|message| message.0.as_str())
          .collect::<Vec<_>>()
          .join(", "))
        .collect::<Vec<_>>()
        .join(", ")
    )));
  }

  let program: Program = parser
    .parse_program()
    .map_err(|error| error.into_diagnostic(&handler).emit())
    .expect("Failed to parse TypeScript module");

  Ok(TypeScriptSource {
    comments,
    program,
    source_map,
  })
}
