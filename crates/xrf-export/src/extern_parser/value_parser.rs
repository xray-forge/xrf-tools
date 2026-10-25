use std::collections::BTreeMap;
use std::path::Path;

use xrf_error::XrfResult;
use xrf_typescript::swc_common::{SourceMap, Spanned};
use xrf_typescript::swc_ecma_ast::{Expr, MemberExpr, MemberProp, Program, TsFnOrConstructorType, TsType};
use xrf_typescript::{TypeScriptSymbol, TypeScriptSymbolResolver};

use super::callable_parser::ExternCallableParser;
use super::diagnostics::{invalid_at, unsupported_export_value_reason};
use super::type_renderer::canonical_type;
use crate::extern_documentation::ExternDocumentation;
use crate::extern_export::ExternExport;
use crate::extern_value::ExternValue;

/// Parses one value passed to an extern declaration.
pub struct ExternValueParser<'a> {
  callable_parser: ExternCallableParser<'a>,
  source_file: &'a Path,
  source_map: &'a SourceMap,
  source_path: &'a str,
  symbol_resolver: &'a TypeScriptSymbolResolver,
}

impl<'a> ExternValueParser<'a> {
  /// Create a parser for one declaration source file.
  pub fn new(
    source_map: &'a SourceMap,
    source_file: &'a Path,
    source_path: &'a str,
    symbol_resolver: &'a TypeScriptSymbolResolver,
  ) -> Self {
    Self {
      callable_parser: ExternCallableParser::new(source_map, source_path),
      source_file,
      source_map,
      source_path,
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

        Ok(ExternExport::Value(ExternValue {
          doc: documentation,
          source: self.source_path.into(),
          type_name: canonical_type(assertion.type_ann.as_ref(), self.source_map),
        }))
      }
      _ => Err(invalid_at(
        self.source_map,
        value.span().lo,
        self.source_path,
        &unsupported_export_value_reason(value, export_name, self.source_map),
      )),
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
      .resolve_symbol(self.source_file, self.source_map, program, local_name)?
      .ok_or_else(|| self.invalid_value(value, export_name))?;

    match symbol {
      TypeScriptSymbol::Callable(signature) => Ok(ExternExport::Callable(self.callable_parser.convert_signature(
        signature,
        documentation,
        parameter_docs,
      ))),
      value => Ok(ExternExport::Value(ExternValue {
        doc: documentation,
        source: self.source_path.into(),
        type_name: value.value_type().expect("Non-callable symbols have value types"),
      })),
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
    let Expr::Ident(object) = member.obj.as_ref() else {
      return Err(self.invalid_value(value, export_name));
    };
    let MemberProp::Ident(property) = &member.prop else {
      return Err(self.invalid_value(value, export_name));
    };
    let type_name: String = self
      .symbol_resolver
      .resolve_member_type(
        self.source_file,
        self.source_map,
        program,
        object.sym.as_ref(),
        property.sym.as_ref(),
      )?
      .ok_or_else(|| self.invalid_value(value, export_name))?;

    Ok(ExternExport::Value(ExternValue {
      doc: documentation,
      source: self.source_path.into(),
      type_name,
    }))
  }

  /// Build a location-aware diagnostic for an unsupported extern value.
  fn invalid_value(&self, value: &Expr, export_name: &str) -> xrf_error::XrfError {
    invalid_at(
      self.source_map,
      value.span().lo,
      self.source_path,
      &unsupported_export_value_reason(value, export_name, self.source_map),
    )
  }
}
