use std::collections::BTreeMap;

use xrf_error::XrfResult;
use xrf_typescript::swc_common::Spanned;
use xrf_typescript::swc_ecma_ast::{ArrowExpr, Pat, TsFnParam, TsFnType};
use xrf_typescript::{TypeScriptFunctionParameter, TypeScriptFunctionSignature, TypeScriptTypeRenderer};

use crate::manifest::{ExternCallable, ExternDocumentation, ExternParameter};
use crate::parse::declaration::extern_diagnostic::ExternDiagnostic;

/// Builds canonical extern callables from TypeScript function contracts.
pub(crate) struct ExternCallableParser<'a> {
  diagnostic: ExternDiagnostic<'a>,
}

impl<'a> ExternCallableParser<'a> {
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
      returns: TypeScriptTypeRenderer::render_annotation(
        arrow.return_type.as_deref(),
        self.diagnostic.get_source_map(),
      ),
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

      params.push(Self::convert_parameter(
        TypeScriptFunctionParameter {
          name: binding.id.sym.to_string(),
          optional: binding.id.optional,
          type_name: TypeScriptTypeRenderer::render_annotation(
            binding.type_ann.as_deref(),
            self.diagnostic.get_source_map(),
          ),
        },
        parameter_docs,
      ));
    }

    Ok(ExternCallable {
      doc: documentation,
      params,
      returns: TypeScriptTypeRenderer::render_canonical(
        &function_type.type_ann.type_ann,
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
    ExternCallable {
      doc: documentation,
      params: signature
        .params
        .into_iter()
        .map(|parameter| Self::convert_parameter(parameter, parameter_docs))
        .collect(),
      returns: signature.returns,
      source: self.diagnostic.get_source_path().into(),
    }
  }

  /// Parse one parameter in an inline arrow function, refusing a pattern no contract can be read from.
  fn parse_parameter(&self, parameter: &Pat, parameter_docs: &BTreeMap<String, String>) -> XrfResult<ExternParameter> {
    let Some(signature) = TypeScriptFunctionParameter::from_pattern(parameter, self.diagnostic.get_source_map()) else {
      let reason: &str = match parameter {
        Pat::Rest(_) => "rest parameters must use explicitly typed identifiers",
        Pat::Assign(_) => {
          "callable extern parameters with defaults must use explicitly typed identifiers or array patterns"
        }
        _ => "callable extern parameters must use explicitly typed identifiers or array patterns",
      };

      return Err(self.diagnostic.invalid_at(parameter.span().lo, reason));
    };

    Ok(Self::convert_parameter(signature, parameter_docs))
  }

  /// A signature's parameter as the manifest records it, with its documentation.
  fn convert_parameter(
    parameter: TypeScriptFunctionParameter,
    parameter_docs: &BTreeMap<String, String>,
  ) -> ExternParameter {
    ExternParameter {
      doc: parameter_docs.get(&parameter.name).cloned(),
      name: parameter.name,
      optional: parameter.optional.then_some(true),
      type_name: parameter.type_name,
    }
  }
}
