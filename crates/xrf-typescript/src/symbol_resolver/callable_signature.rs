use super::typescript_function_parameter::TypeScriptFunctionParameter;
use super::typescript_function_signature::TypeScriptFunctionSignature;
use crate::ast::canonical_ts_type_to_string;
use crate::swc_common::SourceMap;
use crate::swc_ecma_ast::{ArrowExpr, Function, Pat};

/// Convert a function declaration into its manifest-ready signature.
pub fn function_signature(function: &Function, source_map: &SourceMap) -> TypeScriptFunctionSignature {
  TypeScriptFunctionSignature {
    params: function
      .params
      .iter()
      .map(|parameter| parameter_signature(&parameter.pat, source_map))
      .collect(),
    returns: function
      .return_type
      .as_ref()
      .map(|annotation| canonical_type(annotation.type_ann.as_ref(), source_map))
      .unwrap_or_else(|| String::from("unknown")),
  }
}

/// Convert an arrow function declaration into its manifest-ready signature.
pub fn arrow_signature(arrow: &ArrowExpr, source_map: &SourceMap) -> TypeScriptFunctionSignature {
  TypeScriptFunctionSignature {
    params: arrow
      .params
      .iter()
      .map(|parameter| parameter_signature(parameter, source_map))
      .collect(),
    returns: arrow
      .return_type
      .as_ref()
      .map(|annotation| canonical_type(annotation.type_ann.as_ref(), source_map))
      .unwrap_or_else(|| String::from("unknown")),
  }
}

/// Render a callable contract as a TypeScript function type.
pub fn function_type(signature: &TypeScriptFunctionSignature) -> String {
  let params: String = signature
    .params
    .iter()
    .map(|parameter| {
      let optional: &str = if parameter.optional { "?" } else { "" };

      format!("{}{}: {}", parameter.name, optional, parameter.type_name)
    })
    .collect::<Vec<String>>()
    .join(", ");

  format!("({params}) => {}", signature.returns)
}

/// Convert one function pattern into its manifest-ready parameter.
fn parameter_signature(pattern: &Pat, source_map: &SourceMap) -> TypeScriptFunctionParameter {
  match pattern {
    Pat::Ident(binding) => TypeScriptFunctionParameter {
      name: binding.id.sym.to_string(),
      optional: binding.id.optional,
      type_name: binding
        .type_ann
        .as_ref()
        .map(|annotation| canonical_type(annotation.type_ann.as_ref(), source_map))
        .unwrap_or_else(|| String::from("unknown")),
    },
    Pat::Assign(assign) => {
      let mut parameter: TypeScriptFunctionParameter = parameter_signature(assign.left.as_ref(), source_map);
      parameter.optional = true;

      parameter
    }
    Pat::Rest(rest) => {
      let mut parameter: TypeScriptFunctionParameter = parameter_signature(rest.arg.as_ref(), source_map);
      parameter.type_name = format!("...{}", parameter.type_name);

      parameter
    }
    _ => TypeScriptFunctionParameter {
      name: String::from("parameter"),
      optional: false,
      type_name: String::from("unknown"),
    },
  }
}

/// Render a type annotation without failing resolution for unsupported syntax.
fn canonical_type(type_annotation: &crate::swc_ecma_ast::TsType, source_map: &SourceMap) -> String {
  canonical_ts_type_to_string(type_annotation, source_map).unwrap_or_else(|_| String::from("unknown"))
}
