use std::borrow::Cow;
use std::default::Default;
use std::path::Path;
use std::rc::Rc;

use swc_common::comments::SingleThreadedComments;
use swc_common::sync::Lrc;
use swc_common::{SourceFile, SourceMap, Spanned};
use swc_ecma_ast::Program;
use swc_ecma_parser::error::Error as SyntaxError;
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
/// Caller-supplied input decides both failures, so neither aborts: a source that cannot be read is
/// an `XrfError::Io` and any syntax diagnostic an `XrfError::Parsing` naming the position it holds.
pub fn parse_typescript_file(path: &Path) -> XrfResult<TypeScriptSource> {
  let source_map: Lrc<SourceMap> = Default::default();
  let source_file: Rc<SourceFile> = source_map.load_file(path).map_err(|error| {
    XrfError::new_io_error(
      format!("Failed to read TypeScript file {}: {error}.", format_path(path)),
      error.kind(),
    )
  })?;
  let comments: SingleThreadedComments = Default::default();

  let lexer: Lexer = Lexer::new(
    Syntax::Typescript(Default::default()),
    Default::default(),
    StringInput::from(source_file.as_ref()),
    Some(&comments),
  );
  let mut parser: Parser<Lexer> = Parser::new_from(lexer);

  // A diagnostic exists only once the parser has run, so the fatal one is read before the recovered ones.
  let program: Program = parser
    .parse_program()
    .map_err(|error| new_syntax_error(&source_map, path, &[error]))?;
  let recovered: Vec<SyntaxError> = parser.take_errors();

  if !recovered.is_empty() {
    return Err(new_syntax_error(&source_map, path, &recovered));
  }

  Ok(TypeScriptSource {
    comments,
    program,
    source_map,
  })
}

/// Describe SWC syntax diagnostics at the source positions they were reported for.
fn new_syntax_error(source_map: &SourceMap, path: &Path, errors: &[SyntaxError]) -> XrfError {
  let path: String = format_path(path).to_string();
  let described: Vec<String> = errors
    .iter()
    .map(|error| {
      let message: Cow<str> = error.kind().msg();
      // SWC terminates some of its messages and not others; the joined line terminates each once.
      let message: &str = message.trim_end_matches('.');

      match source_map.try_lookup_char_pos(error.span().lo) {
        Ok(position) => format!("{path}:{}:{}: {message}", position.line, position.col.0 + 1),
        Err(_) => format!("{path}: {message}"),
      }
    })
    .collect();

  XrfError::new_parsing_error(format!("Failed to parse TypeScript {}.", described.join("; ")))
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::io::ErrorKind;
  use std::path::{Path, PathBuf};

  use swc_ecma_ast::Program;
  use xrf_error::XrfError;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::parse_typescript_file;

  fn write_source(name: &str, source: &str) -> PathBuf {
    let path: PathBuf = build_absolute_generated_test_resource_path(&format!("typescript-parser/{name}"));

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, source).unwrap();

    path
  }

  fn parse_failure(path: &Path) -> XrfError {
    match parse_typescript_file(path) {
      Ok(_) => panic!("Expected parsing to fail"),
      Err(error) => error,
    }
  }

  #[test]
  fn parses_a_valid_module() {
    let path: PathBuf = write_source("valid.ts", "export const value: number = 1;");

    assert!(matches!(
      parse_typescript_file(&path).unwrap().program,
      Program::Module(_)
    ));
  }

  #[test]
  fn reports_a_source_that_cannot_be_read() {
    let path: PathBuf = build_absolute_generated_test_resource_path("typescript-parser/missing.ts");

    match parse_failure(&path) {
      XrfError::Io { message, kind } => {
        assert_eq!(kind, ErrorKind::NotFound);
        assert!(message.starts_with("Failed to read TypeScript file "), "{message}");
      }
      error => panic!("Expected an IO error, got: {error}"),
    }
  }

  #[test]
  fn reports_a_directory_supplied_as_a_source() {
    let path: PathBuf = write_source("directory/source.ts", "export {};");

    assert!(matches!(parse_failure(path.parent().unwrap()), XrfError::Io { .. }));
  }

  #[test]
  fn reports_a_fatal_syntax_error_with_its_position() {
    let path: PathBuf = write_source("fatal.ts", "export {};\nconst broken: = ;");

    match parse_failure(&path) {
      XrfError::Parsing { message } => assert!(message.contains("fatal.ts:2:15: Unexpected token"), "{message}"),
      error => panic!("Expected a parsing error, got: {error}"),
    }
  }

  #[test]
  fn reports_a_recovered_syntax_error() {
    // Recovered rather than fatal: the parser yields a program and reports the diagnostic only afterwards.
    let path: PathBuf = write_source("recovered.ts", "export {};\nfunction broken(value?: number = 1) {}");

    match parse_failure(&path) {
      XrfError::Parsing { message } => assert!(
        message.contains("recovered.ts:2:17: Parameter cannot have question mark and initializer."),
        "{message}"
      ),
      error => panic!("Expected a parsing error, got: {error}"),
    }
  }

  #[test]
  fn reports_every_recovered_syntax_error_once() {
    let path: PathBuf = write_source(
      "recovered-many.ts",
      "export {};\nclass Broken { abstract run(): void {} }",
    );

    match parse_failure(&path) {
      XrfError::Parsing { message } => {
        assert_eq!(message.matches("recovered-many.ts:2:").count(), 2, "{message}");
        // A message SWC already terminated is not terminated twice.
        assert!(message.contains("an implementation; "), "{message}");
      }
      error => panic!("Expected a parsing error, got: {error}"),
    }
  }
}
