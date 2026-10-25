use std::collections::BTreeMap;
use std::path::Path;

use xrf_error::XrfResult;
use xrf_typescript::swc_ecma_ast::{Expr, MemberExpr, MemberProp, Program, TsFnOrConstructorType, TsType};
use xrf_typescript::{TypeScriptSymbol, TypeScriptSymbolResolver};

use crate::manifest::{ExternDocumentation, ExternExport, ExternValue};
use crate::parse::declaration::extern_callable_parser::ExternCallableParser;
use crate::parse::declaration::extern_diagnostic::ExternDiagnostic;
use crate::parse::declaration::extern_type_renderer::ExternTypeRenderer;

/// Parses one value passed to an extern declaration.
pub(crate) struct ExternValueParser<'a> {
  callable_parser: ExternCallableParser<'a>,
  diagnostic: ExternDiagnostic<'a>,
  source_file: &'a Path,
  symbol_resolver: &'a TypeScriptSymbolResolver,
}

impl<'a> ExternValueParser<'a> {
  /// Create a parser for one declaration source file.
  pub fn new(
    diagnostic: ExternDiagnostic<'a>,
    source_file: &'a Path,
    symbol_resolver: &'a TypeScriptSymbolResolver,
  ) -> Self {
    Self {
      callable_parser: ExternCallableParser::new(diagnostic),
      diagnostic,
      source_file,
      symbol_resolver,
    }
  }

  /// Parse one extern value into its canonical manifest representation.
  pub fn parse(
    &self,
    program: &Program,
    value: &Expr,
    export_name: &str,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> XrfResult<ExternExport> {
    match value {
      Expr::Paren(parenthesized) => self.parse(
        program,
        parenthesized.expr.as_ref(),
        export_name,
        documentation,
        parameter_docs,
      ),
      Expr::Arrow(arrow) => Ok(ExternExport::Callable(self.callable_parser.parse_arrow(
        arrow,
        documentation,
        parameter_docs,
      )?)),
      Expr::Ident(identifier) => self.parse_symbol_reference(
        program,
        identifier.sym.as_ref(),
        value,
        export_name,
        documentation,
        parameter_docs,
      ),
      Expr::Member(member) => self.parse_member_reference(program, member, value, export_name, documentation),
      Expr::TsAs(assertion) => {
        if let TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(function_type)) =
          assertion.type_ann.as_ref()
        {
          return Ok(ExternExport::Callable(self.callable_parser.parse_function_type(
            function_type,
            documentation,
            parameter_docs,
          )?));
        }

        Ok(self.new_value(
          documentation,
          ExternTypeRenderer::render(assertion.type_ann.as_ref(), self.diagnostic.get_source_map()),
        ))
      }
      _ => Err(self.diagnostic.invalid_value(value, export_name)),
    }
  }

  /// Build an extern from a referenced symbol's declared TypeScript contract.
  fn parse_symbol_reference(
    &self,
    program: &Program,
    local_name: &str,
    value: &Expr,
    export_name: &str,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> XrfResult<ExternExport> {
    let symbol: TypeScriptSymbol = self
      .symbol_resolver
      .resolve_symbol(self.source_file, self.diagnostic.get_source_map(), program, local_name)?
      .ok_or_else(|| self.diagnostic.invalid_value(value, export_name))?;

    match symbol {
      TypeScriptSymbol::Callable(signature) => Ok(ExternExport::Callable(self.callable_parser.convert_signature(
        signature,
        documentation,
        parameter_docs,
      ))),
      value => Ok(self.new_value(
        documentation,
        value.value_type().expect("Non-callable symbols have value types"),
      )),
    }
  }

  /// Build an extern value from a property on a referenced object literal.
  fn parse_member_reference(
    &self,
    program: &Program,
    member: &MemberExpr,
    value: &Expr,
    export_name: &str,
    documentation: Option<ExternDocumentation>,
  ) -> XrfResult<ExternExport> {
    let (Expr::Ident(object), MemberProp::Ident(property)) = (member.obj.as_ref(), &member.prop) else {
      return Err(self.diagnostic.invalid_value(value, export_name));
    };
    let type_name: String = self
      .symbol_resolver
      .resolve_member_type(
        self.source_file,
        self.diagnostic.get_source_map(),
        program,
        object.sym.as_ref(),
        property.sym.as_ref(),
      )?
      .ok_or_else(|| self.diagnostic.invalid_value(value, export_name))?;

    Ok(self.new_value(documentation, type_name))
  }

  /// A value extern of this source.
  fn new_value(&self, documentation: Option<ExternDocumentation>, type_name: String) -> ExternExport {
    ExternExport::Value(ExternValue {
      doc: documentation,
      source: self.diagnostic.get_source_path().into(),
      type_name,
    })
  }
}
