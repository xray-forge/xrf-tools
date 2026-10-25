use super::typescript_function_parameter::TypeScriptFunctionParameter;

/// The TypeScript signature of a resolved callable symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScriptFunctionSignature {
  pub params: Vec<TypeScriptFunctionParameter>,
  pub returns: String,
}
