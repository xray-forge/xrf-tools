use std::collections::BTreeMap;

use xrf_error::XrfResult;
use xrf_typescript::TypeScriptFunctionSignature;
use xrf_typescript::swc_common::{SourceMap, SourceMapper, Spanned};
use xrf_typescript::swc_ecma_ast::{ArrowExpr, Pat, TsFnParam, TsFnType, TsTypeAnn};

use super::diagnostics::invalid_at;
use super::type_renderer::canonical_type;
use crate::extern_callable::ExternCallable;
use crate::extern_documentation::ExternDocumentation;
use crate::extern_parameter::ExternParameter;

/// Builds canonical extern callables from TypeScript function contracts.
pub struct ExternCallableParser<'a> {
  source_map: &'a SourceMap,
  source_path: &'a str,
}

impl<'a> ExternCallableParser<'a> {
  /// Create a callable parser for one declaration source file.
  pub fn new(source_map: &'a SourceMap, source_path: &'a str) -> Self {
    Self {
      source_map,
      source_path,
    }
  }

  /// Parse an inline arrow function declared by an extern.
  pub fn parse_arrow(
    &self,
    arrow: &ArrowExpr,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> XrfResult<ExternCallable> {
    let returns: String = arrow.return_type.as_ref().map_or_else(
      || String::from("unknown"),
      |annotation| canonical_type(annotation.type_ann.as_ref(), self.source_map),
    );
    let mut params: Vec<ExternParameter> = Vec::with_capacity(arrow.params.len());

    for parameter in &arrow.params {
      params.push(self.parse_parameter(parameter, parameter_docs)?);
    }

    Ok(ExternCallable {
      doc: documentation,
      params,
      returns,
      source: self.source_path.into(),
    })
  }

  /// Parse a function type asserted on an extern value.
  pub fn parse_function_type(
    &self,
    function_type: &TsFnType,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> XrfResult<ExternCallable> {
    let mut params: Vec<ExternParameter> = Vec::with_capacity(function_type.params.len());

    for parameter in &function_type.params {
      let TsFnParam::Ident(binding) = parameter else {
        return Err(invalid_at(
          self.source_map,
          function_type.span.lo,
          self.source_path,
          "function type assertions must use named parameters",
        ));
      };
      let name: String = binding.id.sym.to_string();
      let type_name: String = binding.type_ann.as_ref().map_or_else(
        || String::from("unknown"),
        |annotation| canonical_type(annotation.type_ann.as_ref(), self.source_map),
      );
      params.push(ExternParameter {
        doc: parameter_docs.get(&name).cloned(),
        name,
        optional: binding.id.optional.then_some(true),
        type_name,
      });
    }

    Ok(ExternCallable {
      doc: documentation,
      params,
      returns: canonical_type(function_type.type_ann.type_ann.as_ref(), self.source_map),
      source: self.source_path.into(),
    })
  }

  /// Convert a callable contract resolved from another TypeScript source file.
  pub fn convert_signature(
    &self,
    signature: TypeScriptFunctionSignature,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> ExternCallable {
    let params: Vec<ExternParameter> = signature
      .params
      .into_iter()
      .map(|parameter| ExternParameter {
        doc: parameter_docs.get(&parameter.name).cloned(),
        name: parameter.name,
        optional: parameter.optional.then_some(true),
        type_name: parameter.type_name,
      })
      .collect();

    ExternCallable {
      doc: documentation,
      params,
      returns: signature.returns,
      source: self.source_path.into(),
    }
  }

  /// Parse one parameter in an inline arrow function.
  fn parse_parameter(&self, parameter: &Pat, parameter_docs: &BTreeMap<String, String>) -> XrfResult<ExternParameter> {
    let (name, annotation, optional): (String, Option<&TsTypeAnn>, bool) = match parameter {
      Pat::Ident(binding) => (
        binding.id.sym.to_string(),
        binding.type_ann.as_deref(),
        binding.id.optional,
      ),
      Pat::Array(pattern) => (
        pattern_name(self.source_map, pattern.span, "[parameter]"),
        pattern.type_ann.as_deref(),
        pattern.optional,
      ),
      Pat::Rest(pattern) => {
        let Pat::Ident(binding) = pattern.arg.as_ref() else {
          return Err(invalid_at(
            self.source_map,
            parameter.span().lo,
            self.source_path,
            "rest parameters must use explicitly typed identifiers",
          ));
        };
        (
          binding.id.sym.to_string(),
          pattern.type_ann.as_deref().or(binding.type_ann.as_deref()),
          false,
        )
      }
      Pat::Assign(pattern) => match pattern.left.as_ref() {
        Pat::Ident(binding) => (binding.id.sym.to_string(), binding.type_ann.as_deref(), true),
        Pat::Array(binding) => (
          pattern_name(self.source_map, binding.span, "[parameter]"),
          binding.type_ann.as_deref(),
          true,
        ),
        _ => {
          return Err(invalid_at(
            self.source_map,
            parameter.span().lo,
            self.source_path,
            "callable extern parameters with defaults must use explicitly typed identifiers or array patterns",
          ));
        }
      },
      _ => {
        return Err(invalid_at(
          self.source_map,
          parameter.span().lo,
          self.source_path,
          "callable extern parameters must use explicitly typed identifiers or array patterns",
        ));
      }
    };

    Ok(ExternParameter {
      doc: parameter_docs.get(&name).cloned(),
      name,
      optional: optional.then_some(true),
      type_name: annotation.map_or_else(
        || String::from("unknown"),
        |value| canonical_type(value.type_ann.as_ref(), self.source_map),
      ),
    })
  }
}

/// Return a readable name for a destructured parameter pattern.
fn pattern_name(source_map: &SourceMap, span: xrf_typescript::swc_common::Span, fallback: &str) -> String {
  let snippet: String = source_map.span_to_snippet(span).unwrap_or_else(|_| fallback.into());
  let mut nesting: usize = 0;

  for (index, character) in snippet.char_indices() {
    match character {
      '[' | '(' | '{' => nesting += 1,
      ']' | ')' | '}' => nesting = nesting.saturating_sub(1),
      ':' if nesting == 0 => return snippet[..index].trim_end().into(),
      _ => {}
    }
  }

  snippet.trim_end().into()
}
