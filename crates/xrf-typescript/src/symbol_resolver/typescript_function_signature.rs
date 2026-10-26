use super::typescript_function_parameter::TypeScriptFunctionParameter;
use crate::TypeScriptTypeRenderer;
use crate::swc_common::SourceMap;
use crate::swc_ecma_ast::{ArrowExpr, Function, Pat, TsTypeAnn};

/// The TypeScript signature of a resolved callable symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScriptFunctionSignature {
  pub params: Vec<TypeScriptFunctionParameter>,
  pub returns: String,
}

impl TypeScriptFunctionSignature {
  /// The signature a function declaration or expression states.
  pub fn from_function(function: &Function, source_map: &SourceMap) -> Self {
    Self::new(
      function.params.iter().map(|parameter| &parameter.pat),
      function.return_type.as_deref(),
      source_map,
    )
  }

  /// The signature an arrow function states.
  pub fn from_arrow(arrow: &ArrowExpr, source_map: &SourceMap) -> Self {
    Self::new(arrow.params.iter(), arrow.return_type.as_deref(), source_map)
  }

  /// The signature as a TypeScript function type: `(name?: Type) => Returns`.
  pub fn type_name(&self) -> String {
    let params: Vec<String> = self.params.iter().map(TypeScriptFunctionParameter::contract).collect();

    format!("({}) => {}", params.join(", "), self.returns)
  }

  /// A signature that names every parameter it can read and keeps the place of one it cannot.
  fn new<'a>(patterns: impl Iterator<Item = &'a Pat>, return_type: Option<&TsTypeAnn>, source_map: &SourceMap) -> Self {
    Self {
      params: patterns
        .map(|pattern| {
          TypeScriptFunctionParameter::from_pattern(pattern, source_map)
            .unwrap_or_else(TypeScriptFunctionParameter::new_unreadable)
        })
        .collect(),
      returns: TypeScriptTypeRenderer::render_annotation(return_type, source_map),
    }
  }
}
