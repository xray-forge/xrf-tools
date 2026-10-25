use std::collections::BTreeMap;

use xrf_error::XrfResult;
use xrf_typescript::TypeScriptFunctionSignature;
use xrf_typescript::swc_common::{Span, Spanned};
use xrf_typescript::swc_ecma_ast::{ArrowExpr, Pat, TsFnParam, TsFnType, TsTypeAnn};

use crate::manifest::{ExternCallable, ExternDocumentation, ExternParameter};
use crate::parse::declaration::extern_diagnostic::ExternDiagnostic;
use crate::parse::declaration::extern_type_renderer::ExternTypeRenderer;

/// Builds canonical extern callables from TypeScript function contracts.
pub(crate) struct ExternCallableParser<'a> {
  diagnostic: ExternDiagnostic<'a>,
}

impl<'a> ExternCallableParser<'a> {
  /// What a destructured parameter is named where its source cannot be read back.
  const PATTERN_NAME: &'static str = "[parameter]";

  /// Create a callable parser for one declaration source file.
  pub fn new(diagnostic: ExternDiagnostic<'a>) -> Self {
    Self { diagnostic }
  }

  /// Parse an inline arrow function declared by an extern.
  pub fn parse_arrow(
    &self,
    arrow: &ArrowExpr,
    documentation: Option<ExternDocumentation>,
    parameter_docs: &BTreeMap<String, String>,
  ) -> XrfResult<ExternCallable> {
    let mut params: Vec<ExternParameter> = Vec::with_capacity(arrow.params.len());

    for parameter in &arrow.params {
      params.push(self.parse_parameter(parameter, parameter_docs)?);
    }

    Ok(ExternCallable {
      doc: documentation,
      params,
      returns: ExternTypeRenderer::render_annotation(arrow.return_type.as_deref(), self.diagnostic.get_source_map()),
      source: self.diagnostic.get_source_path().into(),
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
        return Err(self.diagnostic.invalid_at(
          function_type.span.lo,
          "function type assertions must use named parameters",
        ));
      };
      let name: String = binding.id.sym.to_string();

      params.push(ExternParameter {
        doc: parameter_docs.get(&name).cloned(),
        name,
        optional: binding.id.optional.then_some(true),
        type_name: ExternTypeRenderer::render_annotation(binding.type_ann.as_deref(), self.diagnostic.get_source_map()),
      });
    }

    Ok(ExternCallable {
      doc: documentation,
      params,
      returns: ExternTypeRenderer::render(
        function_type.type_ann.type_ann.as_ref(),
        self.diagnostic.get_source_map(),
      ),
      source: self.diagnostic.get_source_path().into(),
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
      source: self.diagnostic.get_source_path().into(),
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
        self.get_pattern_name(pattern.span),
        pattern.type_ann.as_deref(),
        pattern.optional,
      ),
      Pat::Rest(pattern) => {
        let Pat::Ident(binding) = pattern.arg.as_ref() else {
          return Err(self.diagnostic.invalid_at(
            parameter.span().lo,
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
        Pat::Array(binding) => (self.get_pattern_name(binding.span), binding.type_ann.as_deref(), true),
        _ => {
          return Err(self.diagnostic.invalid_at(
            parameter.span().lo,
            "callable extern parameters with defaults must use explicitly typed identifiers or array patterns",
          ));
        }
      },
      _ => {
        return Err(self.diagnostic.invalid_at(
          parameter.span().lo,
          "callable extern parameters must use explicitly typed identifiers or array patterns",
        ));
      }
    };

    Ok(ExternParameter {
      doc: parameter_docs.get(&name).cloned(),
      name,
      optional: optional.then_some(true),
      type_name: ExternTypeRenderer::render_annotation(annotation, self.diagnostic.get_source_map()),
    })
  }

  /// A readable name for a destructured parameter: its pattern as written, up to the type annotation.
  fn get_pattern_name(&self, span: Span) -> String {
    let snippet: String = self.diagnostic.get_snippet(span, Self::PATTERN_NAME);
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
}
