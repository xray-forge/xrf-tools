use std::collections::BTreeMap;
use std::path::Path;

use xrf_error::XrfResult;
use xrf_typescript::TypeScriptSymbolResolver;
use xrf_typescript::swc_common::comments::Comments;
use xrf_typescript::swc_common::{SourceMap, Spanned};
use xrf_typescript::swc_ecma_ast::{
  Callee, Expr, ExprOrSpread, Lit, ModuleItem, ObjectLit, Program, Prop, PropName, Stmt,
};

use crate::manifest::ExternDocumentation;
use crate::parse::declaration::extern_diagnostic::ExternDiagnostic;
use crate::parse::declaration::extern_statement::ExternStatement;
use crate::parse::declaration::extern_value_parser::ExternValueParser;
use crate::parse::declaration::js_doc_parser::JsDocParser;
use crate::parse::parsed_extern::ParsedExtern;

/// Extracts canonical extern declarations from one parsed TypeScript program.
pub(crate) struct ExternDeclarationParser<'a> {
  diagnostic: ExternDiagnostic<'a>,
  jsdoc_parser: JsDocParser<'a>,
  value_parser: ExternValueParser<'a>,
}

impl<'a> ExternDeclarationParser<'a> {
  /// The function a declaration calls, `xray16/lib`'s `extern`.
  pub const CALLEE: &'static str = "extern";

  /// Create a parser whose diagnostics and documentation refer to one source file.
  pub fn new(
    source_map: &'a SourceMap,
    comments: &'a dyn Comments,
    source_file: &'a Path,
    source_path: &'a str,
    symbol_resolver: &'a TypeScriptSymbolResolver,
  ) -> Self {
    let diagnostic: ExternDiagnostic = ExternDiagnostic::new(source_map, source_path);

    Self {
      diagnostic,
      jsdoc_parser: JsDocParser::new(comments),
      value_parser: ExternValueParser::new(diagnostic, source_file, symbol_resolver),
    }
  }

  /// Every top-level `extern(...)` call of `program`, in source order.
  pub fn parse(&self, program: &Program) -> XrfResult<Vec<ExternStatement>> {
    let statements: Vec<&Stmt> = match program {
      Program::Module(module) => module
        .body
        .iter()
        .filter_map(|item| match item {
          ModuleItem::Stmt(statement) => Some(statement),
          ModuleItem::ModuleDecl(_) => None,
        })
        .collect(),
      Program::Script(script) => script.body.iter().collect(),
    };
    let mut declarations: Vec<ExternStatement> = Vec::new();

    for statement in statements {
      if let Some(declaration) = self.parse_statement(program, statement)? {
        declarations.push(declaration);
      }
    }

    Ok(declarations)
  }

  /// The call a statement makes, where it is an `extern(...)` call at all.
  fn parse_statement(&self, program: &Program, statement: &Stmt) -> XrfResult<Option<ExternStatement>> {
    let Stmt::Expr(statement) = statement else {
      return Ok(None);
    };
    let Expr::Call(call) = statement.expr.as_ref() else {
      return Ok(None);
    };

    if !Self::is_extern_callee(&call.callee) {
      return Ok(None);
    }

    if call.args.len() != 2 {
      return Err(self.diagnostic.invalid_at(
        statement.span.lo,
        "expected a literal name and exactly one exported value",
      ));
    }

    let key: String = Self::get_string_argument(&call.args[0]).ok_or_else(|| {
      self
        .diagnostic
        .invalid_at(call.args[0].expr.span().lo, "extern names must be string literals")
    })?;
    let documentation: Option<ExternDocumentation> = self.jsdoc_parser.parse(statement.span.lo);
    let parameter_docs: BTreeMap<String, String> = self.jsdoc_parser.parameter_docs(statement.span.lo);
    let location = self.diagnostic.span_location(statement.span);
    let is_table: bool = matches!(call.args[1].expr.as_ref(), Expr::Object(_));
    let externs: Vec<ParsedExtern> = match call.args[1].expr.as_ref() {
      Expr::Object(object) => self.parse_object(program, &key, object, documentation, &parameter_docs)?,
      value => vec![ParsedExtern {
        export: self
          .value_parser
          .parse(program, value, &key, documentation, &parameter_docs)?,
        location: location.clone(),
        name: key.clone(),
      }],
    };

    Ok(Some(ExternStatement {
      externs,
      is_table,
      key,
      location,
    }))
  }

  /// Parse each named property in an object-form extern declaration.
  fn parse_object(
    &self,
    program: &Program,
    key: &str,
    object: &ObjectLit,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> XrfResult<Vec<ParsedExtern>> {
    let mut externs: Vec<ParsedExtern> = Vec::with_capacity(object.props.len());

    for property in &object.props {
      let Some(Prop::KeyValue(property)) = property.as_prop().map(Box::as_ref) else {
        return Err(
          self
            .diagnostic
            .invalid_at(property.span().lo, "object externs must contain property assignments"),
        );
      };
      let property_name: String = Self::get_property_name(&property.key).ok_or_else(|| {
        self.diagnostic.invalid_at(
          property.key.span().lo,
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
      let name: String = format!("{key}.{property_name}");

      externs.push(ParsedExtern {
        export: self.value_parser.parse(
          program,
          property.value.as_ref(),
          &name,
          property_doc,
          property_parameter_docs,
        )?,
        location: self.diagnostic.span_location(property.span()),
        name,
      });
    }

    Ok(externs)
  }

  /// Whether a call names `extern` itself, unqualified.
  fn is_extern_callee(callee: &Callee) -> bool {
    let Callee::Expr(callee) = callee else {
      return false;
    };

    matches!(callee.as_ref(), Expr::Ident(identifier) if identifier.sym == *Self::CALLEE)
  }

  /// An argument's value where it is a string literal.
  fn get_string_argument(argument: &ExprOrSpread) -> Option<String> {
    let Expr::Lit(Lit::Str(literal)) = argument.expr.as_ref() else {
      return None;
    };

    Some(literal.value.to_string_lossy().to_string())
  }

  /// An object property's name, which the extern schema takes only as an identifier or a string literal.
  fn get_property_name(name: &PropName) -> Option<String> {
    match name {
      PropName::Ident(value) => Some(value.sym.to_string()),
      PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
      _ => None,
    }
  }
}
