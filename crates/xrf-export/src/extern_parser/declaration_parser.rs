use std::collections::BTreeMap;
use std::path::Path;

use xrf_error::XrfResult;
use xrf_typescript::TypeScriptSymbolResolver;
use xrf_typescript::ast::{expression_callee_name, expression_string_argument};
use xrf_typescript::swc_common::comments::Comments;
use xrf_typescript::swc_common::{SourceMap, Spanned};
use xrf_typescript::swc_ecma_ast::{Expr, ModuleItem, Program, Prop, PropName, Stmt};

use super::diagnostics::{invalid_at, source_span_location};
use super::jsdoc_parser::JsDocParser;
use super::value_parser::ExternValueParser;
use crate::extern_manifest::{ExternDocumentation, ExternSourceLocation, ParsedExtern};

const EXTERN_EXPRESSION: &str = "extern";

/// Extracts canonical extern declarations from one parsed TypeScript program.
pub struct ExternDeclarationParser<'a> {
  source_map: &'a SourceMap,
  jsdoc_parser: JsDocParser<'a>,
  source_path: &'a str,
  value_parser: ExternValueParser<'a>,
}

impl<'a> ExternDeclarationParser<'a> {
  /// Create a parser whose diagnostics and documentation refer to one source file.
  pub fn new(
    source_map: &'a SourceMap,
    comments: &'a dyn Comments,
    source_file: &'a Path,
    source_path: &'a str,
    symbol_resolver: &'a TypeScriptSymbolResolver,
  ) -> Self {
    Self {
      source_map,
      jsdoc_parser: JsDocParser::new(comments),
      source_path,
      value_parser: ExternValueParser::new(source_map, source_file, source_path, symbol_resolver),
    }
  }

  /// Extract all supported top-level extern declarations from `program`.
  pub fn parse(&self, program: &Program) -> XrfResult<Vec<ParsedExtern>> {
    let mut declarations: Vec<ParsedExtern> = Vec::new();

    match program {
      Program::Module(module) => {
        for item in &module.body {
          let ModuleItem::Stmt(statement) = item else {
            continue;
          };
          self.parse_statement(program, statement, &mut declarations)?;
        }
      }
      Program::Script(script) => {
        for statement in &script.body {
          self.parse_statement(program, statement, &mut declarations)?;
        }
      }
    }

    Ok(declarations)
  }

  fn parse_statement(
    &self,
    program: &Program,
    statement: &Stmt,
    declarations: &mut Vec<ParsedExtern>,
  ) -> XrfResult<()> {
    let Stmt::Expr(statement) = statement else {
      return Ok(());
    };
    let Expr::Call(call) = statement.expr.as_ref() else {
      return Ok(());
    };
    if expression_callee_name(&call.callee).as_deref() != Some(EXTERN_EXPRESSION) {
      return Ok(());
    }

    if call.args.len() != 2 {
      return Err(invalid_at(
        self.source_map,
        statement.span.lo,
        self.source_path,
        "expected a literal name and exactly one exported value",
      ));
    }
    let name: String = expression_string_argument(&call.args[0]).ok_or_else(|| {
      invalid_at(
        self.source_map,
        call.args[0].expr.span().lo,
        self.source_path,
        "extern names must be string literals",
      )
    })?;
    let documentation: Option<ExternDocumentation> = self.jsdoc_parser.parse(statement.span.lo);
    let parameter_docs: BTreeMap<String, String> = self.jsdoc_parser.parameter_docs(statement.span.lo);
    let location: ExternSourceLocation = source_span_location(self.source_map, statement.span, self.source_path);

    match call.args[1].expr.as_ref() {
      Expr::Object(object) => {
        self.parse_object(program, &name, object, documentation, &parameter_docs, declarations)?
      }
      value => declarations.push(ParsedExtern {
        export: self
          .value_parser
          .parse(program, value, &name, documentation, &parameter_docs)?,
        location,
        name,
      }),
    }

    Ok(())
  }

  /// Parse each named property in an object-form extern declaration.
  fn parse_object(
    &self,
    program: &Program,
    name: &str,
    object: &xrf_typescript::swc_ecma_ast::ObjectLit,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
    declarations: &mut Vec<ParsedExtern>,
  ) -> XrfResult<()> {
    for property in &object.props {
      let Prop::KeyValue(property) = property
        .as_prop()
        .ok_or_else(|| {
          invalid_at(
            self.source_map,
            property.span().lo,
            self.source_path,
            "object externs must contain property assignments",
          )
        })?
        .as_ref()
      else {
        return Err(invalid_at(
          self.source_map,
          property.span().lo,
          self.source_path,
          "object externs must contain property assignments",
        ));
      };

      let property_name: String = property_name(&property.key).ok_or_else(|| {
        invalid_at(
          self.source_map,
          property.key.span().lo,
          self.source_path,
          "object extern property names must be identifiers or string literals",
        )
      })?;

      let property_doc: Option<ExternDocumentation> = self
        .jsdoc_parser
        .parse(property.span().lo)
        .or_else(|| documentation.clone());
      let own_parameter_docs: BTreeMap<String, String> = self.jsdoc_parser.parameter_docs(property.span().lo);
      let property_parameter_docs: &BTreeMap<String, String> = if own_parameter_docs.is_empty() {
        parameter_docs
      } else {
        &own_parameter_docs
      };
      let export_name: String = format!("{name}.{property_name}");

      declarations.push(ParsedExtern {
        export: self.value_parser.parse(
          program,
          property.value.as_ref(),
          &export_name,
          property_doc,
          property_parameter_docs,
        )?,
        location: source_span_location(self.source_map, property.span(), self.source_path),
        name: export_name,
      });
    }

    Ok(())
  }
}

/// Return an object property name supported by the extern schema.
fn property_name(name: &PropName) -> Option<String> {
  match name {
    PropName::Ident(value) => Some(value.sym.to_string()),
    PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
    _ => None,
  }
}
